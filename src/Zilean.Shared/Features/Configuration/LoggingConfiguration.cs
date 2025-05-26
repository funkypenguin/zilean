namespace Zilean.Shared.Features.Configuration;

public static class LoggingConfiguration
{
    private const string DefaultLoggingContents =
        """
        {
          "Serilog": {
            "MinimumLevel": {
              "Default": "Information",
              "Override": {
                "Microsoft": "Warning",
                "System": "Warning",
                "System.Net.Http.HttpClient.Scraper.LogicalHandler": "Warning",
                "System.Net.Http.HttpClient.Scraper.ClientHandler": "Warning",
                "Microsoft.AspNetCore.Hosting.Diagnostics": "Error",
                "Microsoft.AspNetCore.DataProtection": "Error",
                "Zilean.Database.Services.Lucene.ImdbLuceneMatchingService": "Information",
                "Zilean.Database.Services.FuzzyString.ImdbFuzzyStringMatchingService": "Information",
              }
            }
          }
        }
        """;

    public static IConfigurationBuilder AddLoggingConfiguration(this IConfigurationBuilder configuration, string configurationFolderPath)
    {
        var loggingPath = Path.Combine(configurationFolderPath, ConfigurationLiterals.LoggingConfigFilename);

        EnsureExists(loggingPath);

        configuration.AddJsonFile(loggingPath, false, false);

        return configuration;
    }

    private static void EnsureExists(string loggingPath)
    {
        if (File.Exists(loggingPath))
        {
            return;
        }

        File.WriteAllText(loggingPath, DefaultLoggingContents);
    }
}
