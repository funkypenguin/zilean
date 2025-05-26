namespace Zilean.Scraper.Features.Ingestion.Dmm;

public class DmmFileDownloader(ILogger<DmmFileDownloader> logger, ZileanConfiguration configuration)
{
    private const string Filename = "main.zip";

    private static readonly IReadOnlyCollection<string> _filesToIgnore =
    [
        "index.html",
        "404.html",
        "dedupe.sh",
        "CNAME",
    ];

    public async Task<string> DownloadFileToTempPath(DmmLastImport? dmmLastImport, CancellationToken cancellationToken)
    {
        logger.LogInformation("Downloading DMM Hashlists");

        var tempDirectory = Path.Combine(Path.GetTempPath(), "DMMHashlists");

        if (dmmLastImport is not null)
        {
            if (DateTime.UtcNow - dmmLastImport.OccuredAt < TimeSpan.FromMinutes(configuration.Dmm.MinimumReDownloadIntervalMinutes))
            {
                logger.LogInformation(
                    "DMM Hashlists download not required as last download was less than the configured {Minutes} minutes re-download interval set in DMM Configuration",
                    configuration.Dmm.MinimumReDownloadIntervalMinutes);
                return tempDirectory;
            }
        }

        EnsureDirectoryIsClean(tempDirectory);

        var tempZipPath = Path.Combine(tempDirectory, "DMMHashlists.zip");

        using var handler = new HttpClientHandler();
        handler.AutomaticDecompression = DecompressionMethods.None;

        using var client = new HttpClient(handler);
        client.BaseAddress = new(configuration.Dmm.GitHubRepoUrl);
        client.Timeout = TimeSpan.FromMinutes(10);

        client.DefaultRequestHeaders.UserAgent.ParseAdd("curl/7.54");

        if (!string.IsNullOrEmpty(configuration.Dmm.GitHubPat))
        {
            client.DefaultRequestHeaders.Authorization = new("Bearer", configuration.Dmm.GitHubPat);
        }

        await using (var httpStream = await client.GetStreamAsync(Filename, cancellationToken))
        await using (var fileStream = new FileStream(tempZipPath, FileMode.Create, FileAccess.Write, FileShare.None, 8192, true))
        {
            await httpStream.CopyToAsync(fileStream, cancellationToken);
        }

        using var archive = ZipFile.OpenRead(tempZipPath);

        foreach (var entry in archive.Entries)
        {
            if (_filesToIgnore.Contains(entry.Name))
            {
                continue;
            }

            var entryPath = Path.Combine(tempDirectory, Path.GetFileName(entry.FullName));
            Directory.CreateDirectory(Path.GetDirectoryName(entryPath)!);

            // skip directories
            if (string.IsNullOrWhiteSpace(entry.Name))
            {
                continue;
            }

            await using var entryStream = entry.Open();
            await using var outFile = new FileStream(entryPath, FileMode.Create, FileAccess.Write, FileShare.None, 8192, useAsync: true);
            await entryStream.CopyToAsync(outFile, cancellationToken);
        }

        if (File.Exists(tempZipPath))
        {
            File.Delete(tempZipPath);
        }

        logger.LogInformation("Downloaded and extracted Repository to {TempDirectory}", tempDirectory);
        return tempDirectory;
    }


    private static void EnsureDirectoryIsClean(string tempDirectory)
    {
        if (Directory.Exists(tempDirectory))
        {
            Directory.Delete(tempDirectory, true);
        }

        Directory.CreateDirectory(tempDirectory);
    }
}
