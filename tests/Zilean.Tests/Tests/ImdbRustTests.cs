using Zilean.Proto.RustServer;
using Zilean.Shared.Features.Grpc;
using Zilean.Shared.Features.Torrents;
using TorrentInfo = Zilean.Shared.Features.Torrents.TorrentInfo;

namespace Zilean.Tests.Tests;

public class ImdbRustTests(ITestOutputHelper output)
{
    private readonly ILogger<ZileanRustServer.ZileanRustServerClient> _logger = Substitute.For<ILogger<ZileanRustServer.ZileanRustServerClient>>();
    private readonly ITorrentInfoService _infoService = Substitute.For<ITorrentInfoService>();

    private readonly ZileanConfiguration _zileanConfiguration = new()
    {
        Database = new()
        {
            ConnectionString = "Host=localhost;Database=zilean;Username=postgres;Password=postgres;Include Error Detail=true;Timeout=30;CommandTimeout=3600;",
        },
    };

    [Fact]
    public async Task MatchTorrent_TV_Success()
    {
        var stopwatch = new Stopwatch();

        var count = 100;

        var torrents = GenerateTvTorrents(count).ToDictionary(
            torrent => torrent.InfoHash!,
            torrent => torrent);

        var imdbMatchingService = new RustGrpcService(_logger, _zileanConfiguration, _infoService);

        await imdbMatchingService.StartServer();

        var results = new List<SearchImdbResponse>();

        stopwatch.Start();

        foreach (var torrent in torrents)
        {
            var result = await imdbMatchingService.SearchImdbData(new()
            {
                Category = "tvSeries",
                Title = torrent.Value.Filename.ToLower(),
            });

            results.Add(result);
        }

        await imdbMatchingService.StopServer();

        var elapsed = stopwatch.Elapsed;
        output.WriteLine($"Parsed {count} torrents in {elapsed.TotalSeconds} seconds");

        foreach (var result in results)
        {
            output.WriteLine($"Result: {result.Matches.ElementAtOrDefault(0)?.Title} ({result.Matches.ElementAtOrDefault(0)?.Year}) - {result.Matches.ElementAtOrDefault(0)?.ImdbId}");
        }
    }

    [Fact]
    public async Task MatchTorrent_Movie_Success()
    {
        var stopwatch = new Stopwatch();

        var count = 10000;

        var torrents = GenerateMovieTorrents(count).ToDictionary(
            torrent => torrent.InfoHash!,
            torrent => torrent);

        var imdbMatchingService = new RustGrpcService(_logger, _zileanConfiguration, _infoService);

        await imdbMatchingService.StartServer();

        var results = new List<SearchImdbResponse>();

        stopwatch.Start();

        foreach (var torrent in torrents)
        {
            var result = await imdbMatchingService.SearchImdbData(new()
            {
                Category = "movie",
                Title = torrent.Value.Filename.ToLower(),
            });

            results.Add(result);
        }

        await imdbMatchingService.StopServer();

        var elapsed = stopwatch.Elapsed;
        output.WriteLine($"Parsed {count} torrents in {elapsed.TotalSeconds} seconds");

        foreach (var result in results)
        {
            output.WriteLine($"Result: {result.Matches.ElementAtOrDefault(0)?.Title} ({result.Matches.ElementAtOrDefault(0)?.Year}) - {result.Matches.ElementAtOrDefault(0)?.ImdbId}");
        }
    }

    [Fact]
    public async Task MatchTorrentFilename_Movie_Success()
    {
        var stopwatch = new Stopwatch();

        var count = 1000;

        var torrents = GenerateMovieTorrentsFilenames(count).ToDictionary(
            torrent => torrent.InfoHash!,
            torrent => torrent);

        var imdbMatchingService = new RustGrpcService(_logger, _zileanConfiguration, _infoService);

        await imdbMatchingService.StartServer();

        var results = new List<TorrentInfo>();

        stopwatch.Start();

        await imdbMatchingService.ParseAndPopulateAsync(torrents, results, count);

        await imdbMatchingService.StopServer();

        var elapsed = stopwatch.Elapsed;
        output.WriteLine($"Parsed {count} torrents in {elapsed.TotalSeconds} seconds");

        results.Should().NotBeNull().And.NotBeEmpty();
        results.Should().AllBeOfType<TorrentInfo>();
        results.Count.Should().Be(count);

        foreach (var torrentInfo in results)
        {
            output.WriteLine($"Raw: {torrentInfo.RawTitle} -> Parsed: {torrentInfo.ParsedTitle} ({torrentInfo.Year})");
        }
    }

    [Fact]
    public async Task IngestImdbData_DoesNotThrow()
    {
        var imdbMatchingService = new RustGrpcService(_logger, _zileanConfiguration, _infoService);

        var exception = await Record.ExceptionAsync(async () =>
        {
            await imdbMatchingService.StartServer();
            await imdbMatchingService.IngestImdbData(new()
            {
                ForceIndex = true,
            });
            await imdbMatchingService.StopServer();
        });

        Assert.Null(exception);
    }

    [Fact]
    public async Task StartAndStopServer_DoesNotThrow()
    {
        var imdbMatchingService = new RustGrpcService(_logger, _zileanConfiguration, _infoService);

        var exception = await Record.ExceptionAsync(async () =>
        {
            await imdbMatchingService.StartServer();
            await Task.Delay(1000);
            await imdbMatchingService.StopServer();
        });

        Assert.Null(exception);
    }

    private static List<ExtractedDmmEntry> GenerateMovieTorrents(int count)
    {
        var torrents = new List<ExtractedDmmEntry>();
        var random = new Random();
        var titles = new[]
        {
            "iron man",
            "the dark knight",
            "inception",
            "the matrix",
            "the terminator",
        };

        for (int i = 0; i < count; i++)
        {
            var infoHash = $"1234562828797{i:D4}";
            var filename = titles[random.Next(titles.Length)];
            var filesize = (long)(random.NextDouble() * 100000000000);

            torrents.Add(new ExtractedDmmEntry(infoHash, filename, filesize, null));
        }

        return torrents;
    }

    private static List<ExtractedDmmEntry> GenerateTvTorrents(int count)
    {
        var torrents = new List<ExtractedDmmEntry>();
        var random = new Random();

        var titles = new[]
        {
            "the witcher",
            "star trek discovery",
            "the mandalorian",
            "breaking bad",
            "supernatural",
        };

        for (int i = 0; i < count; i++)
        {
            var infoHash = $"1234562828797{i:D4}";
            var filename = titles[random.Next(titles.Length)];
            var filesize = (long)(random.NextDouble() * 100000000000);

            torrents.Add(new ExtractedDmmEntry(infoHash, filename, filesize, null));
        }

        return torrents;
    }

    private static List<ExtractedDmmEntry> GenerateMovieTorrentsFilenames(int count)
    {
        var torrents = new List<ExtractedDmmEntry>();
        var random = new Random();
        var titles = new[]
        {
            "Iron.Man.2008.INTERNAL.REMASTERED.2160p.UHD.BluRay.X265-IAMABLE",
            "Harry.Potter.and.the.Sorcerers.Stone.2001.2160p.UHD.BluRay.X265-IAMABLE",
            "The.Dark.Knight.2008.2160p.UHD.BluRay.X265-IAMABLE",
            "Inception.2010.2160p.UHD.BluRay.X265-IAMABLE",
            "The.Matrix.1999.2160p.UHD.BluRay.X265-IAMABLE"
        };

        for (int i = 0; i < count; i++)
        {
            var infoHash = $"1234562828797{i:D4}";
            var filename = titles[random.Next(titles.Length)];
            var filesize = (long)(random.NextDouble() * 100000000000);

            torrents.Add(new ExtractedDmmEntry(infoHash, filename, filesize, null));
        }

        return torrents;
    }

    private static List<ExtractedDmmEntry> GenerateTvTorrentsFilenames(int count)
    {
        var torrents = new List<ExtractedDmmEntry>();
        var random = new Random();

        var titles = new[]
        {
            "The.Witcher.S01E01.1080p.WEB.H264-METCON",
            "The.Witcher.S01E02.1080p.WEB.H264-METCON",
            "The.Witcher.S01E03.1080p.WEB.H264-METCON",
            "The.Witcher.S01E04.1080p.WEB.H264-METCON",
            "The.Witcher.S01E05.1080p.WEB.H264-METCON",
        };

        for (int i = 0; i < count; i++)
        {
            var infoHash = $"1234562828797{i:D4}";
            var filename = titles[random.Next(titles.Length)];
            var filesize = (long)(random.NextDouble() * 100000000000);

            torrents.Add(new ExtractedDmmEntry(infoHash, filename, filesize, null));
        }

        return torrents;
    }
}
