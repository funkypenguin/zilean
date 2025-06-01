namespace Zilean.Shared.Features.Configuration;

public class ParsingConfiguration
{
    public int IngestionBatchSize { get; set; } = 100;
    public int ParsingBatchSize { get; set; } = 100;
    public int ParsingThreads { get; set; } = 4;
    public int MatchingBatchSize { get; set; } = 100;
    public int StorageBatchSize { get; set; } = 5000;
}
