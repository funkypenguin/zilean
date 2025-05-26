namespace Zilean.Scraper.Features.Commands;

public class ResyncImdbCommand : BaseCommand
{
    private readonly ITorrentInfoService _torrentInfoService;
    private readonly IRustGrpcService _rustGrpcService;
    private readonly ZileanDbContext _dbContext;
    private readonly IServiceProvider _serviceProvider;
    private readonly ILogger<ResyncImdbCommand> _logger;

    public ResyncImdbCommand(
        ITorrentInfoService torrentInfoService,
        IRustGrpcService rustGrpcService,
        ZileanDbContext dbContext,
        IServiceProvider serviceProvider,
        ILogger<ResyncImdbCommand> logger) : base("resync-imdb", "Resync IMDB data and optionally retag torrents")
    {
        _torrentInfoService = torrentInfoService;
        _rustGrpcService = rustGrpcService;
        _dbContext = dbContext;
        _serviceProvider = serviceProvider;
        _logger = logger;

        AddForceDownload();
        AddForceCreateIndex();
        AddRetagMissingImdbsOption();
        AddRetagAllImdbsOption();
    }

    private void AddRetagAllImdbsOption()
    {
        var retagAllImdbsOption = new Option<bool>("-a", "--retag-all-imdbs")
        {
            Description = "Will attempt to match IMDB ids for all torrents.",
            DefaultValueFactory = _ => false,
        };
        Options.Add(retagAllImdbsOption);
    }

    private void AddRetagMissingImdbsOption()
    {
        var retagMissingImdbsOption = new Option<bool>("-t", "--retag-missing-imdbs")
        {
            Description = "Will attempt to match IMDB ids for anything that is missing them in the database.",
            DefaultValueFactory = _ => false,
        };
        Options.Add(retagMissingImdbsOption);
    }

    private void AddForceDownload()
    {
        var forceDownloadOption = new Option<bool>("-d", "--force-download")
        {
            Description = "Skip the date check on imdb imports (last 30 days) and force downloads a new file.",
            DefaultValueFactory = _ => false,
        };

        Options.Add(forceDownloadOption);
    }

    private void AddForceCreateIndex()
    {
        var forceCreateIndex = new Option<bool>("-i", "--force-create-index")
        {
            Description = "Force the creation of the index file, even if it exists.",
            DefaultValueFactory = _ => false,
        };

        Options.Add(forceCreateIndex);
    }

    protected override async Task<int> ExecuteAsync(ParseResult parseResult, CancellationToken cancellationToken)
    {
        var settings = ResyncImdbCommandSettings.Parse(parseResult);

        if (settings is {RetagAllImdbs: true, RetagMissingImdbs: true})
        {
            _logger.LogError("Cannot use both --retag-missing-imdbs and --retag-all-imdbs at the same time");
            return 1;
        }

        var result = 0;

        await _rustGrpcService.IngestImdbData(
            new()
            {
                ForceDownload = settings.ForceDownload,
                ForceIndex = settings.ForceCreateIndex,
            });

        try
        {
            if (settings.RetagMissingImdbs)
            {
                await HandleRetagging(all: false);
                return 0;
            }

            if (settings.RetagAllImdbs)
            {
                await HandleRetagging(all: true);
                return 0;
            }
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Error occurred during ResyncImdbCommand");
            result = 1;
        }

        return result;
    }

    private async Task HandleRetagging(bool all = false)
    {
        var torrents = _dbContext.Torrents.AsNoTracking()
            .Where(x => x.Category != "xxx");

        if (!all)
        {
            torrents = torrents.Where(x => x.ImdbId == null);
        }

        var processableTorrents = await torrents.ToListAsync();

        _logger.LogInformation("Found {TorrentCount} torrents", processableTorrents.Count);

        if (processableTorrents.Count > 0)
        {
            _logger.LogInformation("Starting to process torrents...");

            await _rustGrpcService.StartServer();

            var updatedTorrents = await _rustGrpcService.MatchImdbIdsForBatchAsync(processableTorrents);

            await _rustGrpcService.StopServer();

            _logger.LogInformation("Updating {TorrentCount} torrents", updatedTorrents.Count);

            await using var scope = _serviceProvider.CreateAsyncScope();
            var scopedDbContext = scope.ServiceProvider.GetRequiredService<ZileanDbContext>();

            scopedDbContext.AttachRange(updatedTorrents);
            scopedDbContext.UpdateRange(updatedTorrents);
            await scopedDbContext.SaveChangesAsync();

            _logger.LogInformation("Finished processing torrents");
        }
        else
        {
            _logger.LogInformation("No torrents found to match");
        }

        await _torrentInfoService.VaccumTorrentsIndexes(CancellationToken.None);
    }

    private class ResyncImdbCommandSettings
    {
        public bool ForceDownload { get; init; }
        public bool ForceCreateIndex { get; init; }
        public bool RetagMissingImdbs { get; init; }
        public bool RetagAllImdbs { get; init; }

        public static ResyncImdbCommandSettings Parse(ParseResult parseResult) => new()
        {
            ForceDownload = parseResult.GetValue<bool>("-d"),
            ForceCreateIndex = parseResult.GetValue<bool>("-i"),
            RetagMissingImdbs = parseResult.GetValue<bool>("-t"),
            RetagAllImdbs = parseResult.GetValue<bool>("-a"),
        };
    }
}
