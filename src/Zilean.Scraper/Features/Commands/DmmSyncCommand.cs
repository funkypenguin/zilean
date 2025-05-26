namespace Zilean.Scraper.Features.Commands;

public sealed class DmmSyncCommand(DmmScraping dmmScraping) : BaseCommand("dmm-sync", "Synchronize DMM torrents with the database")
{
    protected override Task<int> ExecuteAsync(ParseResult parseResult, CancellationToken cancellationToken) =>
        dmmScraping.Execute(CancellationToken.None);
}
