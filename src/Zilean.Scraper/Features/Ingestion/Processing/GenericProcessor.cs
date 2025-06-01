using Zilean.Shared.Features.Torrents;

namespace Zilean.Scraper.Features.Ingestion.Processing;

public abstract class GenericProcessor<TInput>(
    ILoggerFactory loggerFactory,
    ITorrentInfoService torrentInfoService,
    IRustGrpcService rustGrpcService,
    ZileanConfiguration configuration)
    where TInput : class
{
    private Channel<Dictionary<string, ExtractedDmmEntry>>? _parseChannel;
    private Channel<List<TorrentInfo>>? _matchingChannel;
    private Channel<List<TorrentInfo>>? _storeChannel;
    private readonly ObjectPool<Dictionary<string, ExtractedDmmEntry>> _torrentsListPool = new DefaultObjectPoolProvider().Create<Dictionary<string, ExtractedDmmEntry>>();
    private readonly ObjectPool<List<TorrentInfo>> _torrentsProcessedPool = new DefaultObjectPoolProvider().Create<List<TorrentInfo>>();
    private HashSet<string> _blacklistedHashes = [];

    protected readonly ILogger<GenericProcessor<TInput>> _logger = loggerFactory.CreateLogger<GenericProcessor<TInput>>();
    protected abstract ExtractedDmmEntry TransformToTorrent(TInput input);
    protected readonly ProcessedCounts _processedCounts = new();
    protected readonly ZileanConfiguration _configuration = configuration;

    protected async Task ProcessAsync(Func<ChannelWriter<Task<TInput>>, CancellationToken, Task> producerAction, CancellationToken cancellationToken)
    {
        await rustGrpcService.StartServer();

        _blacklistedHashes = await torrentInfoService.GetBlacklistedItems();

        var torrentChannel = Channel.CreateBounded<Task<TInput>>(new BoundedChannelOptions(_configuration.Parsing.IngestionBatchSize)
        {
            SingleReader = true,
            SingleWriter = false,
            FullMode = BoundedChannelFullMode.Wait,
        });

        _parseChannel = Channel.CreateBounded<Dictionary<string, ExtractedDmmEntry>>(new BoundedChannelOptions(_configuration.Parsing.ParsingBatchSize)
        {
            SingleWriter = false,
            SingleReader = true,
        });

        _matchingChannel = Channel.CreateBounded<List<TorrentInfo>>(new BoundedChannelOptions(_configuration.Parsing.MatchingBatchSize)
        {
            SingleWriter = false,
            SingleReader = true,
        });

        _storeChannel = Channel.CreateBounded<List<TorrentInfo>>(new BoundedChannelOptions(_configuration.Parsing.StorageBatchSize)
        {
            SingleWriter = false,
            SingleReader = true,
            FullMode = BoundedChannelFullMode.Wait,
        });

        var producerTask = producerAction(torrentChannel.Writer, cancellationToken);
        var consumerTask = ConsumeAsync(torrentChannel.Reader, cancellationToken);
        var parseConsumerTask = ParseConsumerAsync(_parseChannel.Reader, cancellationToken);
        var matchConsumerTask = MatchingConsumerAsync(_matchingChannel.Reader, cancellationToken);
        var storeConsumerTask = StoreConsumerAsync(_storeChannel.Reader, cancellationToken);

        await Task.WhenAll(producerTask, consumerTask);

        await CompleteParseChannelAsync(parseConsumerTask);

        await CompleteMatchingChannelAsync(matchConsumerTask);

        await CompleteStoreChannelAsync(storeConsumerTask);

        await rustGrpcService.StopServer();
    }

    private async Task CompleteParseChannelAsync(Task parseConsumerTask)
    {
        _parseChannel.Writer.Complete();
        _logger.LogInformation("Waiting for parse consumer to finish");
        await parseConsumerTask;
    }

    private async Task CompleteMatchingChannelAsync(Task matchingConsumerTask)
    {
        _matchingChannel.Writer.Complete();
        _logger.LogInformation("Waiting for matching consumer to finish");
        await matchingConsumerTask;
    }

    private async Task CompleteStoreChannelAsync(Task storeConsumerTask)
    {
        _storeChannel.Writer.Complete();
        _logger.LogInformation("Waiting for store consumer to finish");
        await storeConsumerTask;
    }

    private async Task ConsumeAsync(ChannelReader<Task<TInput>> reader, CancellationToken cancellationToken)
    {
        var batch = new List<Task<TInput>>(_configuration.Parsing.IngestionBatchSize);

        try
        {
            await foreach (var task in reader.ReadAllAsync(cancellationToken))
            {
                batch.Add(task);

                if (batch.Count < _configuration.Parsing.IngestionBatchSize)
                {
                    continue;
                }

                await OnProcessTorrentsAsync(batch, cancellationToken);
                batch.Clear();
            }
        }
        catch (OperationCanceledException)
        {
            _logger.LogInformation("Processing cancelled, attempting to flush remaining batch");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Unexpected error during processing, attempting to flush remaining batch");
        }
        finally
        {
            if (batch.Count > 0)
            {
                try
                {
                    _logger.LogInformation("Final flush of {Count} remaining items in batch", batch.Count);
                    await OnProcessTorrentsAsync(batch, cancellationToken);
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, "Error during final batch flush");
                }
            }
        }
    }

    private async Task OnProcessTorrentsAsync(List<Task<TInput>> batch, CancellationToken cancellationToken)
    {
        var torrents = _torrentsListPool.Get();

        try
        {
            await foreach (var result in Task.WhenEach(batch).WithCancellation(cancellationToken))
            {
                var current = await result;
                var extractedTorrent = TransformToTorrent(current);
                torrents.TryAdd(extractedTorrent.InfoHash, extractedTorrent);
            }

            if (torrents.Count == 0 || cancellationToken.IsCancellationRequested)
            {
                return;
            }

            torrents.FilterBlacklistedTorrents(_blacklistedHashes, _logger, _processedCounts);

            await _parseChannel.Writer.WriteAsync(torrents, cancellationToken);
        }
        catch (OperationCanceledException)
        {
            _logger.LogInformation("Processing cancelled");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error processing batch of torrents. Batch size: {BatchSize}", batch.Count);
        }
    }

    private async Task ParseConsumerAsync(ChannelReader<Dictionary<string, ExtractedDmmEntry>> reader, CancellationToken cancellationToken)
    {
        await foreach (var torrents in reader.ReadAllAsync(cancellationToken))
        {
            var torrentInfos = _torrentsProcessedPool.Get();
            try
            {
                _logger.LogDebug("Parsing {Count} torrents", torrents.Count);
                await rustGrpcService.ParseAndPopulateAsync(torrents, torrentInfos, _configuration.Parsing.ParsingBatchSize);

                if (torrentInfos.Count == 0 || cancellationToken.IsCancellationRequested)
                {
                    _logger.LogDebug("No torrents to match after parsing, skipping batch");
                    _torrentsProcessedPool.Return(torrentInfos);
                    return;
                }

                if (_matchingChannel is not null && torrentInfos.Count > 0)
                {
                    await _matchingChannel.Writer.WriteAsync(torrentInfos, cancellationToken);
                }
            }
            catch (Exception e)
            {
                _logger.LogError(e, "Error parsing torrents. Batch size: {BatchSize}", torrents.Count);
                torrentInfos.Clear();
                _torrentsProcessedPool.Return(torrentInfos);
            }
            finally
            {
                torrents.Clear();
                _torrentsListPool.Return(torrents);
            }
        }
    }

    private async Task MatchingConsumerAsync(ChannelReader<List<TorrentInfo>> reader, CancellationToken cancellationToken)
    {
        await foreach (var matchableTorrents in reader.ReadAllAsync(cancellationToken))
        {
            try
            {
                _logger.LogDebug("Matching {Count} torrents", matchableTorrents.Count);
                await rustGrpcService.MatchImdbIdsForBatchAsync(matchableTorrents, returnQueue: false);

                if (matchableTorrents.Count == 0 || cancellationToken.IsCancellationRequested)
                {
                    _logger.LogInformation("No torrents to store after matching, skipping batch");
                    _torrentsProcessedPool.Return(matchableTorrents);
                    continue;
                }

                await _storeChannel.Writer.WriteAsync(matchableTorrents, cancellationToken);
            }
            catch (Exception e)
            {
                _logger.LogError(e, "Error matching torrents. Batch size: {BatchSize}", matchableTorrents.Count);
                matchableTorrents.Clear();
                _torrentsProcessedPool.Return(matchableTorrents);
            }
        }
    }

    private async Task StoreConsumerAsync(ChannelReader<List<TorrentInfo>> reader, CancellationToken cancellationToken)
    {
        await foreach (var batch in reader.ReadAllAsync(cancellationToken))
        {
            try
            {
                _logger.LogDebug("Storing batch of {Count} torrents", batch.Count);
                await torrentInfoService.BulkCopyTorrentsAsync(batch, cancellationToken);
                _processedCounts.AddProcessed(batch.Count);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Failed to store batch of {Count} torrents", batch.Count);
                batch.Clear();
                _torrentsProcessedPool.Return(batch);
            }
            finally
            {
                batch.Clear();
                _torrentsProcessedPool.Return(batch);
            }
        }
    }
}
