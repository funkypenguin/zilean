namespace Zilean.Database.Services;

public interface IImdbFileService
{
    Task<ImdbSearchResult[]> SearchForImdbIdAsync(string query, int? year = null, string? category = null);
}
