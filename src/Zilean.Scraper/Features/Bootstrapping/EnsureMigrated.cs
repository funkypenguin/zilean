namespace Zilean.Scraper.Features.Bootstrapping;

public class EnsureMigrated(ILogger<EnsureMigrated> logger, ZileanDbContext dbContext) : IHostedService
{
    public async Task StartAsync(CancellationToken cancellationToken)
    {
        logger.LogInformation("Applying Migrations...");
        await dbContext.Database.MigrateAsync(cancellationToken: cancellationToken);
        logger.LogInformation("Migrations Applied");
    }

    public Task StopAsync(CancellationToken cancellationToken) => Task.CompletedTask;
}
