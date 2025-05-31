namespace Zilean.Shared.Features.Configuration;

public class DmmConfiguration
{
    public bool EnableScraping { get; set; } = true;
    public bool EnableEndpoint { get; set; } = true;
    public string ScrapeSchedule { get; set; } = "0 * * * *";
    public int MinimumReDownloadIntervalMinutes { get; set; } = 30;
    public int MaxFilteredResults { get; set; } = 200;
    public double MinimumScoreMatch { get; set; } = 0.85;
    public string? GitHubPat { get; set; } = null;
    public string GitHubRepoUrl { get; set; } = "https://github.com/debridmediamanager/hashlists/zipball/main/";
}
