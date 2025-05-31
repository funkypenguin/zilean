using Lucene.Net.Documents;
using Lucene.Net.Index;
using Lucene.Net.Search;

namespace Zilean.Database.Services.Lucene;

public class ImdbLuceneMatchingService(ILogger<ImdbLuceneMatchingService> logger, ZileanConfiguration configuration) : IImdbMatchingService
{
    private readonly ObjectPool<ConcurrentDictionary<string, string?>> _imdbCache = new DefaultObjectPoolProvider().Create<ConcurrentDictionary<string, string?>>();
    private LuceneSession? _imdbFilesIndex;

    public async Task PopulateImdbData() => _imdbFilesIndex = await IndexImdbDocumentsInMemory();

    public void DisposeImdbData()
    {
        _imdbFilesIndex?.Writer.Dispose();
        _imdbFilesIndex?.Directory.Dispose();
        _imdbFilesIndex?.Dispose();
    }

    public async Task<ConcurrentQueue<TorrentInfo>?> MatchImdbIdsForBatchAsync(IEnumerable<TorrentInfo> batch, bool returnQueue = true)
    {
        if (_imdbFilesIndex is null)
        {
            throw new InvalidOperationException("IMDb data has not been loaded yet.");
        }

        var imdbCache = _imdbCache.Get();

        try
        {
            var parallelOptions = new ParallelOptions
            {
                MaxDegreeOfParallelism = configuration.Imdb.UseAllCores switch
                {
                    true => Environment.ProcessorCount,
                    false => configuration.Imdb.NumberOfCores,
                },
            };

            var updatedTorrents = new ConcurrentQueue<TorrentInfo>();

            var groupedByYearAndCategory = batch.GroupBy(t => new
            {
                t.Year,
                t.Category,
            });

            using var reader = _imdbFilesIndex.Writer.GetReader(applyAllDeletes: true);
            var searcher = new IndexSearcher(reader);


            await Parallel.ForEachAsync(
                groupedByYearAndCategory, parallelOptions, async (torrentGroup, _) =>
                {
                    foreach (var torrent in torrentGroup)
                    {
                        if (imdbCache.TryGetValue(torrent.CacheKey(), out var imdbId))
                        {
                            torrent.ImdbId = imdbId;
                            continue;
                        }

                        var bestMatch = GetBestMatch(torrent, searcher); // remains sync unless changed

                        if (bestMatch == null)
                        {
                            logger.NoSuitableMatchFound(torrent.NormalizedTitle, torrent.Category);
                            continue;
                        }

                        if (bestMatch.ImdbId != torrent.ImdbId)
                        {
                            logger.TorrentUpdated(
                                torrent.NormalizedTitle,
                                torrent.ImdbId,
                                bestMatch.ImdbId,
                                bestMatch.Score,
                                torrent.Category,
                                bestMatch.Title,
                                bestMatch.Year);

                            torrent.ImdbId = bestMatch.ImdbId;
                            imdbCache[torrent.CacheKey()] = bestMatch.ImdbId;

                            if (returnQueue)
                            {
                                updatedTorrents.Enqueue(torrent);
                            }
                        }
                        else
                        {
                            logger.TorrentRetained(
                                torrent.NormalizedTitle,
                                torrent.ImdbId,
                                bestMatch.Score,
                                torrent.Category,
                                bestMatch.Title,
                                bestMatch.Year);
                        }
                    }

                    await ValueTask.CompletedTask; // required if there's no real async logic
                });


            return returnQueue ? updatedTorrents : null;
        }
        finally
        {
            imdbCache.Clear();
            _imdbCache.Return(imdbCache);
        }
    }

    private BestMatch? GetBestMatch(TorrentInfo torrent, IndexSearcher searcher, int maxResults = 10)
    {
        var matches = MatchTitle(torrent, searcher, maxResults);

        if (!matches.Any())
        {
            return null;
        }

        BestMatch? bestMatch = null;
        double highestScore = 0;

        foreach (var match in matches)
        {
            if (!(match.Score > highestScore))
            {
                continue;
            }

            highestScore = match.Score;
            bestMatch = match;
        }

        return bestMatch;
    }

    private List<BestMatch> MatchTitle(TorrentInfo torrent, IndexSearcher searcher, int maxResults = 3)
    {
        if (string.IsNullOrWhiteSpace(torrent.NormalizedTitle))
        {
            return [];
        }

        var combinedQuery = new BooleanQuery();

        AddTitleAndCategoryToQuery(torrent, combinedQuery);

        if (torrent.Year is > 0)
        {
            AddYearToQuery(torrent, combinedQuery);
        }

        var topDocs = searcher.Search(combinedQuery, maxResults);

        if (topDocs.ScoreDocs.Length == 0)
        {
            return [];
        }

        var results = new List<BestMatch>();

        foreach (var scoreDoc in topDocs.ScoreDocs)
        {
            var doc = searcher.Doc(scoreDoc.Doc);

            var imdbId = doc.Get(LuceneIndexEntry.ImdbId);
            var title = doc.Get(LuceneIndexEntry.Title);
            var year = doc.GetField(LuceneIndexEntry.Year)?.GetInt32Value() ?? 0;

            results.Add(new(imdbId, title, year, scoreDoc.Score));
        }

        return results;
    }

    private static void AddTitleAndCategoryToQuery(TorrentInfo torrent, BooleanQuery combinedQuery)
    {
        var query = new BooleanQuery();

        var fuzzyTitleQuery = new FuzzyQuery(new(LuceneIndexEntry.Title, torrent.NormalizedTitle), 2, 1, 1, false);
        query.Add(fuzzyTitleQuery, Occur.MUST);

        var categoryQuery = new TermQuery(new(LuceneIndexEntry.Category, torrent.Category.ToLowerInvariant()));
        query.Add(categoryQuery, Occur.MUST);

        combinedQuery.Add(query, Occur.MUST);
    }

    private static void AddYearToQuery(TorrentInfo torrent, BooleanQuery combinedQuery)
    {
        var yearZeroQuery = new TermQuery(new(LuceneIndexEntry.Year, "0"));
        var yearRangeQuery = NumericRangeQuery.NewInt32Range(LuceneIndexEntry.Year, torrent.Year!.Value-1, torrent.Year!.Value+1, true, true);
        var yearQuery = new BooleanQuery
        {
            { yearZeroQuery, Occur.SHOULD },
            { yearRangeQuery, Occur.SHOULD },
        };

        combinedQuery.Add(yearQuery, Occur.SHOULD);
    }

    private async Task<LuceneSession> IndexImdbDocumentsInMemory()
    {
        var luceneSession = LuceneSession.NewInstance();

        logger.LogInformation("Indexing IMDb entries...");

        await using var sqlConnection = new NpgsqlConnection(configuration.Database.ConnectionString);
        await sqlConnection.OpenAsync();

        var imdbFiles = sqlConnection.Query<ImdbFile>(
            """
            SELECT
                "ImdbId",
                Lower(
                unaccent(
                    regexp_replace(
                        regexp_replace(trim("Title"), '\s+', ' ', 'g'), -- Normalize whitespace
                        '[^\w\s]', '', 'g' -- Remove non-alphanumeric characters but keep spaces
                        )
                    )
                ) AS "Title",
                "Adult",
                "Category",
                "Year"
            FROM
                public."ImdbFiles"
            WHERE
                "Category" IN ('tvSeries', 'tvShort', 'tvMiniSeries', 'tvSpecial', 'movie', 'tvMovie')
            """);

        foreach (var imdb in imdbFiles)
        {
            var doc = new Document
            {
                new StringField(LuceneIndexEntry.ImdbId, imdb.ImdbId, Field.Store.YES),
                new StringField(LuceneIndexEntry.Title, imdb.Title, Field.Store.YES),
                new StringField(LuceneIndexEntry.Category, GetCategory(imdb.Category).ToLowerInvariant(), Field.Store.YES),
                new Int32Field(LuceneIndexEntry.Year, imdb.Year, new FieldType
                {
                    IsStored = true,
                    IsIndexed = true,
                    IsTokenized = false,
                    NumericType = NumericType.INT32,
                    IndexOptions = IndexOptions.DOCS_ONLY,
                }),
            };

            luceneSession.Writer.AddDocument(doc);
        }

        luceneSession.Writer.Flush(triggerMerge: false, applyAllDeletes: false);

        logger.LogInformation("Indexed {Count} IMDb entries", luceneSession.Writer.NumDocs);

        return luceneSession;
    }

    private static string GetCategory(string? imdbCategory) =>
        imdbCategory switch
        {
            "tvSeries" => "tvSeries",
            "tvShort" => "tvSeries",
            "tvMiniSeries" => "tvSeries",
            "tvSpecial" => "tvSeries",
            "tvMovie" => "movie",
            "movie" => "movie",
            _ => "unknown",
        };


    private record BestMatch(string ImdbId, string Title, int Year, double Score);
}
