namespace Zilean.Scraper.Features.Commands;

public abstract class BaseCommand : Command
{
    protected BaseCommand(string name, string description) : base(name, description) => SetAction(ExecuteAsync);

    protected abstract Task<int> ExecuteAsync(ParseResult parseResult, CancellationToken cancellationToken);
}
