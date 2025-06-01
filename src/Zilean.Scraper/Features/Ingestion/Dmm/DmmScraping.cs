using Zilean.Shared.Features.Torrents;

namespace Zilean.Scraper.Features.Ingestion.Dmm;

public class DmmScraping(
    IRustGrpcService rustGrpcService,
    ITorrentInfoService torrentInfoService,
    ILogger<DmmScraping> logger)
{
    public async Task<int> Execute(CancellationToken cancellationToken)
    {
        try
        {
            await rustGrpcService.IngestImdbData(new(), cancellationToken);

            await rustGrpcService.IngestDmmPagesAsync(cancellationToken);

            logger.LogInformation("All files processed");

            await torrentInfoService.VaccumTorrentsIndexes(cancellationToken);

            logger.LogInformation("DMM Internal Tasks Completed");

            return 0;
        }
        catch (TaskCanceledException)
        {
            return 0;
        }
        catch (OperationCanceledException)
        {
            return 0;
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Error occurred during DMM Scraper Task");
            return 1;
        }
    }
}
