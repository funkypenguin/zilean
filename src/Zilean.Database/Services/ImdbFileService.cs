namespace Zilean.Database.Services;

public class ImdbFileService(ILogger<ImdbFileService> logger, ZileanConfiguration configuration) : BaseDapperService(logger, configuration), IImdbFileService
{
    public async Task<ImdbSearchResult[]> SearchForImdbIdAsync(string query, int? year = null, string? category = null) =>
        await ExecuteCommandAsync(async connection =>
        {
            const string sql =
                """
                SELECT
                    imdb_id as "ImdbId",
                    title as "Title",
                    year as "Year",
                    score as "Score",
                    category as "Category"
                FROM search_imdb_meta(@query, @category, @year, 10)
                """;

            var parameters = new DynamicParameters();

            parameters.Add("@query", query);
            parameters.Add("@category", category);
            parameters.Add("@year", year);

            var result = await connection.QueryAsync<ImdbSearchResult>(sql, parameters);

            return result.ToArray();
        }, "Error finding imdb metadata.");
}
