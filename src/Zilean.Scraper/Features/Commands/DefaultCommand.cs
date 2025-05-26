namespace Zilean.Scraper.Features.Commands;

public sealed class DefaultCommand : RootCommand
{
    public DefaultCommand(
        DmmSyncCommand dmmSyncCommand,
        GenericSyncCommand genericSyncCommand,
        ResyncImdbCommand resyncImdbCommand)
    {
        Subcommands.Add(dmmSyncCommand);
        Subcommands.Add(genericSyncCommand);
        Subcommands.Add(resyncImdbCommand);
    }
}
