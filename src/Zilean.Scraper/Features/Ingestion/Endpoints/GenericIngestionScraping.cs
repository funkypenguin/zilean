using Zilean.Shared.Features.Imdb;
using Zilean.Shared.Features.Torrents;

namespace Zilean.Scraper.Features.Ingestion.Endpoints;

public class GenericIngestionScraping(
    ZileanConfiguration configuration,
    ITorrentInfoService torrentInfoService,
    IRustGrpcService rustGrpcService,
    ILoggerFactory loggerFactory,
    IHttpClientFactory clientFactory,
    ILogger<GenericIngestionScraping> logger,
    KubernetesServiceDiscovery kubernetesServiceDiscovery)
{
    public async Task<int> Execute(CancellationToken cancellationToken)
    {
        logger.LogInformation("Starting ingestion scraping");

        List<GenericEndpoint> endpointsToProcess = [];

        await DiscoverUrlsFromKubernetesServices(cancellationToken, endpointsToProcess);

        AddZurgInstancesToUrls(endpointsToProcess);

        AddZileanInstancesToUrls(endpointsToProcess);

        AddGenericInstancesToUrls(endpointsToProcess);

        if (endpointsToProcess.Count == 0)
        {
            logger.LogInformation("No URLs to process, exiting");
            return 0;
        }

        var completedCount = 0;

        var ingestionProcessor = new StreamedEntryProcessor(torrentInfoService, rustGrpcService, loggerFactory, clientFactory, configuration);

        foreach (var endpoint in endpointsToProcess)
        {
            try
            {

                await ingestionProcessor.ProcessEndpointAsync(endpoint, cancellationToken);
                completedCount++;
            }
            catch (OperationCanceledException)
            {
                logger.LogInformation("Ingestion scraping cancelled URL: {@Url}", endpoint);
            }
            catch (Exception ex)
            {
                logger.LogError(ex, "Error processing URL: {@Url}", endpoint);
            }
        }

        await torrentInfoService.VaccumTorrentsIndexes(cancellationToken);

        await rustGrpcService.StopServer();

        logger.LogInformation("Ingestion scraping completed for {Count} URLs", completedCount);

        return 0;
    }

    private void AddGenericInstancesToUrls(List<GenericEndpoint> urlsToProcess)
    {
        if (configuration.Ingestion.GenericInstances.Count > 0)
        {
            logger.LogInformation("Adding Generic instances to the list of URLs to process");
            urlsToProcess.AddRange(configuration.Ingestion.GenericInstances);
        }
    }

    private void AddZileanInstancesToUrls(List<GenericEndpoint> urlsToProcess)
    {
        if (configuration.Ingestion.ZileanInstances.Count > 0)
        {
            logger.LogInformation("Adding Zilean instances to the list of URLs to process");
            urlsToProcess.AddRange(configuration.Ingestion.ZileanInstances);
        }
    }

    private void AddZurgInstancesToUrls(List<GenericEndpoint> urlsToProcess)
    {
        if (configuration.Ingestion.ZurgInstances.Count > 0)
        {
            logger.LogInformation("Adding Zurg instances to the list of URLs to process");
            urlsToProcess.AddRange(configuration.Ingestion.ZurgInstances);
        }
    }

    private async Task DiscoverUrlsFromKubernetesServices(CancellationToken cancellationToken, List<GenericEndpoint> urlsToProcess)
    {
        if (configuration.Ingestion.Kubernetes.EnableServiceDiscovery)
        {
            logger.LogInformation("Discovering URLs from Kubernetes services");
            var endpoints = await kubernetesServiceDiscovery.DiscoverUrlsAsync(cancellationToken);
            logger.LogInformation("Discovered {Count} URLs from Kubernetes services", endpoints.Count);
            urlsToProcess.AddRange(endpoints);
        }
    }
}
