namespace Zilean.Scraper.Features.Commands;

public class GenericSyncCommand(GenericIngestionScraping genericIngestion) : BaseCommand("generic-sync", "Synchronize generic ingestion")
{
    protected override Task<int> ExecuteAsync(ParseResult parseResult, CancellationToken cancellationToken) =>
        genericIngestion.Execute(CancellationToken.None);
}
