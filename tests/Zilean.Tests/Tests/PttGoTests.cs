// using Zilean.Proto.TorrentParser;
//
// namespace Zilean.Tests.Tests;
//
// public class PttGoTests : IDisposable
// {
//     private readonly ITestOutputHelper _output;
//     private readonly GoTorrentNameService _parseTorrentNameService;
//
//     public PttGoTests(ITestOutputHelper output)
//     {
//         _output = output;
//         var loggerParse = Substitute.For<ILogger<TorrentParser.TorrentParserClient>>();
//         var zileanConfiguration = new ZileanConfiguration();
//         _parseTorrentNameService = new GoTorrentNameService(loggerParse, zileanConfiguration);
//     }
//
//     [Fact]
//     public async Task ParseTorrent_Movie_Success()
//     {
//         var stopwatch = new Stopwatch();
//
//         var count = 100000;
//
//         var torrents = GenerateMovieTorrents(count).ToDictionary(
//             torrent => torrent.InfoHash!,
//             torrent => torrent);
//
//         var output = new List<TorrentInfo>();
//
//         stopwatch.Start();
//         await _parseTorrentNameService.ParseAndPopulateAsync(torrents, output, count);
//         var elapsed = stopwatch.Elapsed;
//         _output.WriteLine($"Parsed {count} torrents in {elapsed.TotalSeconds} seconds");
//
//         output.Should().NotBeNull().And.NotBeEmpty();
//         output.Should().AllBeOfType<TorrentInfo>();
//         output.Count.Should().Be(count);
//
//         foreach (var torrentInfo in output)
//         {
//             _output.WriteLine($"Raw: {torrentInfo.RawTitle} -> Parsed: {torrentInfo.ParsedTitle} ({torrentInfo.Year})");
//         }
//     }
//
//     [Fact]
//     public async Task ParseTorrent_TvSeries_Success()
//     {
//         var stopwatch = new Stopwatch();
//
//         var count = 5000;
//
//         var torrents = GenerateTvTorrents(count).ToDictionary(
//             torrent => torrent.InfoHash!,
//             torrent => torrent);
//
//         var output = new List<TorrentInfo>();
//
//         stopwatch.Start();
//         await _parseTorrentNameService.ParseAndPopulateAsync(torrents, output, count);
//         var elapsed = stopwatch.Elapsed;
//         _output.WriteLine($"Parsed {count} torrents in {elapsed.TotalSeconds} seconds");
//
//         output.Should().NotBeNull().And.NotBeEmpty();
//         output.Should().AllBeOfType<TorrentInfo>();
//         output.Count.Should().Be(count);
//
//         foreach (var torrentInfo in output)
//         {
//             _output.WriteLine($"Raw: {torrentInfo.RawTitle} -> Parsed: {torrentInfo.ParsedTitle} ({torrentInfo.Year})");
//         }
//     }
//
//     private static List<ExtractedDmmEntry> GenerateMovieTorrents(int count)
//     {
//         var torrents = new List<ExtractedDmmEntry>();
//         var random = new Random();
//         var titles = new[]
//         {
//             "Iron.Man.2008.INTERNAL.REMASTERED.2160p.UHD.BluRay.X265-IAMABLE",
//             "Harry.Potter.and.the.Sorcerers.Stone.2001.2160p.UHD.BluRay.X265-IAMABLE",
//             "The.Dark.Knight.2008.2160p.UHD.BluRay.X265-IAMABLE",
//             "Inception.2010.2160p.UHD.BluRay.X265-IAMABLE",
//             "The.Matrix.1999.2160p.UHD.BluRay.X265-IAMABLE"
//         };
//
//         for (int i = 0; i < count; i++)
//         {
//             var infoHash = $"1234562828797{i:D4}";
//             var filename = titles[random.Next(titles.Length)];
//             var filesize = (long)(random.NextDouble() * 100000000000);
//
//             torrents.Add(new ExtractedDmmEntry(infoHash, filename, filesize, null));
//         }
//
//         return torrents;
//     }
//
//     private static List<ExtractedDmmEntry> GenerateTvTorrents(int count)
//     {
//         var torrents = new List<ExtractedDmmEntry>();
//         var random = new Random();
//
//         var titles = new[]
//         {
//             "The.Witcher.S01E01.1080p.WEB.H264-METCON",
//             "The.Witcher.S01E02.1080p.WEB.H264-METCON",
//             "The.Witcher.S01E03.1080p.WEB.H264-METCON",
//             "The.Witcher.S01E04.1080p.WEB.H264-METCON",
//             "The.Witcher.S01E05.1080p.WEB.H264-METCON",
//         };
//
//         for (int i = 0; i < count; i++)
//         {
//             var infoHash = $"1234562828797{i:D4}";
//             var filename = titles[random.Next(titles.Length)];
//             var filesize = (long)(random.NextDouble() * 100000000000);
//
//             torrents.Add(new ExtractedDmmEntry(infoHash, filename, filesize, null));
//         }
//
//         return torrents;
//     }
//
//     public void Dispose() => _parseTorrentNameService.StopServer().GetAwaiter().GetResult();
// }
