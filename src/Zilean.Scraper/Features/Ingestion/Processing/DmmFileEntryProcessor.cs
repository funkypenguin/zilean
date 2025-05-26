using Microsoft.IO;

namespace Zilean.Scraper.Features.Ingestion.Processing;

public partial class DmmFileEntryProcessor(
    DmmService dmmService,
    ITorrentInfoService torrentInfoService,
    IRustGrpcService rustGrpcService,
    ILoggerFactory loggerFactory,
    ZileanConfiguration configuration) : GenericProcessor<ExtractedDmmEntry>(loggerFactory, torrentInfoService, rustGrpcService, configuration)
{
    [GeneratedRegex("""<iframe src="https:\/\/debridmediamanager.com\/hashlist#(.*)"></iframe>""")]
    private static partial Regex HashCollectionMatcher { get; }
    private List<string> _filesToProcess = [];
    private readonly ObjectPool<List<ExtractedDmmEntry>> _torrentsListPool = new DefaultObjectPoolProvider().Create<List<ExtractedDmmEntry>>();
    private static readonly RecyclableMemoryStreamManager _msManager = new();
    private static ReadOnlySpan<byte> FilenameUtf8 => "filename"u8;
    private static ReadOnlySpan<byte> HashUtf8 => "hash"u8;
    private static ReadOnlySpan<byte> BytesUtf8 => "bytes"u8;
    public ConcurrentDictionary<string, int> ExistingPages { get; private set; } = [];
    public ConcurrentDictionary<string, int> NewPages { get; set; } = [];
    protected override ExtractedDmmEntry TransformToTorrent(ExtractedDmmEntry input) => input;

    public async Task ProcessFilesAsync(List<string> files, CancellationToken cancellationToken)
    {
        var sw = Stopwatch.StartNew();
        _filesToProcess = files;
        _processedCounts.Reset();
        _logger.LogInformation("Processing {Count} DMM Files", _filesToProcess.Count);
        await ProcessAsync(ProduceEntriesAsync, cancellationToken);
        _processedCounts.WriteOutput(_configuration, sw, NewPages);
        sw.Stop();
    }

    private async Task ProduceEntriesAsync(ChannelWriter<Task<ExtractedDmmEntry>> writer, CancellationToken cancellationToken)
    {
        foreach (var file in _filesToProcess)
        {
            if (cancellationToken.IsCancellationRequested)
            {
                _logger.LogInformation("Processing cancelled");
                break;
            }

            var fileName = Path.GetFileName(file);
            if (ExistingPages.TryGetValue(fileName, out _) || NewPages.TryGetValue(fileName, out _))
            {
                continue;
            }

            _logger.LogInformation("Processing file: {FileName}", fileName);

            try
            {
                var torrents = await ProcessPageAsync(file, fileName, cancellationToken);
                foreach (var torrent in torrents)
                {
                    await writer.WriteAsync(Task.FromResult(torrent), cancellationToken);
                }
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error processing file: {FileName}", fileName);
            }
        }

        writer.Complete();
    }

    private async Task<List<ExtractedDmmEntry>> ProcessPageAsync(string filePath, string filenameOnly, CancellationToken cancellationToken)
    {
        if (!File.Exists(filePath))
        {
            return [];
        }

        var pageSource = await File.ReadAllTextAsync(filePath, cancellationToken);
        var match = HashCollectionMatcher.Match(pageSource);

        if (!match.Success)
        {
            await AddParsedPage(filenameOnly, 0, cancellationToken);
            return [];
        }

        var torrents = _torrentsListPool.Get();

        try
        {
            if (Decompressor.TryDecompress(match.Groups[1].Value, out string decodedJson))
            {
                await using var stream = _msManager.GetStream();
                await using var writer = new StreamWriter(stream, new UTF8Encoding(encoderShouldEmitUTF8Identifier: false), leaveOpen: true);
                await writer.WriteAsync(decodedJson);
                await writer.FlushAsync(cancellationToken);
                stream.Position = 0;

                var reader = new Utf8JsonReader(new ReadOnlySpan<byte>(stream.GetBuffer(), 0, (int)stream.Length));

                ParseTorrents(ref reader, torrents);

                if (torrents.Count == 0)
                {
                    await AddParsedPage(filenameOnly, 0, cancellationToken);
                    return [];
                }

                var sanitizedTorrents = torrents
                    .Where(x => x.Filesize > 0)
                    .GroupBy(x => x.InfoHash)
                    .Select(group => group.FirstOrDefault())
                    .Where(x => !string.IsNullOrEmpty(x.Filename))
                    .OfType<ExtractedDmmEntry>()
                    .ToList();

                await AddParsedPage(filenameOnly, sanitizedTorrents.Count, cancellationToken);
                return sanitizedTorrents;
            }

            _logger.LogInformation("Failed to decompress data from file: {FilePath}. Skipping Entries...", filePath);

            return [];
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error parsing file: {FilePath}. Skipping Entries...", filePath);
            await AddParsedPage(filenameOnly, 0, cancellationToken);
            return [];
        }
        finally
        {
            torrents.Clear();
            _torrentsListPool.Return(torrents);
        }
    }

    private static ExtractedDmmEntry? ParsePageContent(ref Utf8JsonReader reader)
    {
        ReadOnlySpan<byte> filenameUtf8 = default;
        long filesize = 0;
        ReadOnlySpan<byte> hashUtf8 = default;

        while (reader.Read())
        {
            if (reader.TokenType == JsonTokenType.EndObject)
            {
                break;
            }

            if (reader.TokenType == JsonTokenType.PropertyName)
            {
                var propName = reader.ValueSpan;
                reader.Read();

                if (propName.SequenceEqual(FilenameUtf8))
                {
                    filenameUtf8 = reader.ValueSpan;
                }
                else if (propName.SequenceEqual(BytesUtf8))
                {
                    filesize = reader.GetInt64();
                }
                else if (propName.SequenceEqual(HashUtf8))
                {
                    hashUtf8 = reader.ValueSpan;
                }
            }
        }

        if (filenameUtf8.IsEmpty || hashUtf8.IsEmpty)
        {
            return null;
        }

        string filename = Encoding.UTF8.GetString(filenameUtf8).Replace(".", " ", StringComparison.Ordinal);
        string hash = Encoding.UTF8.GetString(hashUtf8);

        return new(hash, filename, filesize, null);
    }

    public async Task LoadParsedPages(CancellationToken cancellationToken)
    {
        var parsedPages = await dmmService.GetIngestedPagesAsync(cancellationToken);
        if (parsedPages.Count != 0)
        {
#pragma warning disable IDE0306
            ExistingPages = new(parsedPages.ToDictionary(x => x.Page, x => x.EntryCount));
#pragma warning restore IDE0306
        }

        _logger.LogInformation("Loaded {Count} previously parsed pages", ExistingPages.Count);
    }

    private async Task AddParsedPage(string filename, int entryCount, CancellationToken cancellationToken)
    {
        await dmmService.AddPageToIngestedAsync(new()
        {
            EntryCount = entryCount,
            Page = filename
        }, cancellationToken);

        NewPages.TryAdd(filename, entryCount);
    }

    [SuppressMessage("Style", "IDE0010:Add missing cases")]
    private static void ParseTorrents(ref Utf8JsonReader reader, List<ExtractedDmmEntry> torrents)
    {
        if (!reader.Read())
        {
            return;
        }

        // ReSharper disable once SwitchStatementMissingSomeEnumCasesNoDefault
        switch (reader.TokenType)
        {
            case JsonTokenType.StartArray:
                ParseArray(ref reader, torrents);
                return;
            case JsonTokenType.StartObject:
                ParseNestedTorrents(ref reader, torrents);
                break;
        }
    }

    private static void ParseNestedTorrents(ref Utf8JsonReader reader, List<ExtractedDmmEntry> torrents)
    {
        while (reader.Read())
        {
            if (reader.TokenType == JsonTokenType.PropertyName && reader.GetString() == "torrents")
            {
                reader.Read();
                if (reader.TokenType == JsonTokenType.StartArray)
                {
                    ParseArray(ref reader, torrents);
                }
                break;
            }

            if (reader.TokenType == JsonTokenType.EndObject)
            {
                break;
            }
        }
    }

    private static void ParseArray(ref Utf8JsonReader reader, List<ExtractedDmmEntry> torrents)
    {
        while (reader.Read())
        {
            if (reader.TokenType == JsonTokenType.StartObject)
            {
                var entry = ParsePageContent(ref reader);
                if (entry != null)
                {
                    torrents.Add(entry);
                }
            }
            else if (reader.TokenType == JsonTokenType.EndArray)
            {
                break;
            }
        }
    }
}
