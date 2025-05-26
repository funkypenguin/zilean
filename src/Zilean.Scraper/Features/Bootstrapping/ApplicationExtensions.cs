namespace Zilean.Scraper.Features.Bootstrapping;

public static class ApplicationExtensions
{
    public static async Task<int> ExecuteCommandLine(this IHost host, params string[] args)
    {
        var rootCommand = host.Services.GetRequiredService<DefaultCommand>();

        var config = new CommandLineConfiguration(rootCommand)
        {
            EnableDefaultExceptionHandler = true,
        };

        var exitCode = await config.InvokeAsync(args);

        await host.StopAsync().ConfigureAwait(false);

        return exitCode;
    }
}
