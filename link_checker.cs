// link_checker.cs
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using System.Threading;

class LinkChecker
{
    private static readonly string RESET = "\x1b[0m";
    private static readonly string GREEN = "\x1b[92m";
    private static readonly string YELLOW = "\x1b[93m";
    private static readonly string RED = "\x1b[91m";
    private static readonly string GRAY = "\x1b[90m";

    static async Task<int> CheckUrl(HttpClient client, string url, int timeoutSec, bool follow)
    {
        try
        {
            var cts = new CancellationTokenSource(TimeSpan.FromSeconds(timeoutSec));
            var request = new HttpRequestMessage(HttpMethod.Get, url);
            request.Headers.UserAgent.TryParseAdd("LinkChecker/1.0");
            var response = await client.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, cts.Token);
            if (!follow && (response.StatusCode == System.Net.HttpStatusCode.Redirect ||
                            response.StatusCode == System.Net.HttpStatusCode.MovedPermanently ||
                            response.StatusCode == System.Net.HttpStatusCode.Found ||
                            response.StatusCode == System.Net.HttpStatusCode.SeeOther ||
                            response.StatusCode == System.Net.HttpStatusCode.TemporaryRedirect ||
                            response.StatusCode == System.Net.HttpStatusCode.PermanentRedirect))
            {
                // возвращаем статус (останавливаемся)
                return (int)response.StatusCode;
            }
            return (int)response.StatusCode;
        }
        catch
        {
            return -1;
        }
    }

    static List<string> LoadLinks(string filename)
    {
        var lines = File.ReadAllLines(filename);
        return lines.Where(l => !string.IsNullOrWhiteSpace(l) && !l.StartsWith("#")).Select(l => l.Trim()).ToList();
    }

    static async Task Main(string[] args)
    {
        if (args.Length == 0)
        {
            Console.Error.WriteLine("Использование: dotnet run <файл.txt> [--threads N] [--timeout S] [--follow] [--output file]");
            return;
        }
        string source = args[0];
        int threads = 4;
        int timeoutSec = 5;
        bool follow = false;
        string output = null;
        for (int i = 1; i < args.Length; i++)
        {
            if (args[i] == "--threads" && i+1 < args.Length)
                threads = int.Parse(args[++i]);
            else if (args[i] == "--timeout" && i+1 < args.Length)
                timeoutSec = int.Parse(args[++i]);
            else if (args[i] == "--follow")
                follow = true;
            else if (args[i] == "--output" && i+1 < args.Length)
                output = args[++i];
        }

        List<string> links;
        if (source.EndsWith(".txt"))
            links = LoadLinks(source);
        else
            links = args.ToList();

        if (!links.Any())
        {
            Console.Error.WriteLine("Нет ссылок для проверки.");
            return;
        }

        Console.WriteLine($"Проверка {links.Count} ссылок (потоков: {threads}, таймаут: {timeoutSec}с)...");
        var start = DateTime.Now;

        var results = new Dictionary<string, int>();
        var httpClient = new HttpClient();
        var semaphore = new SemaphoreSlim(threads);
        var tasks = new List<Task>();

        foreach (var url in links)
        {
            await semaphore.WaitAsync();
            tasks.Add(Task.Run(async () =>
            {
                try
                {
                    int status = await CheckUrl(httpClient, url, timeoutSec, follow);
                    lock (results) results[url] = status;
                }
                finally
                {
                    semaphore.Release();
                }
            }));
        }
        await Task.WhenAll(tasks);

        var elapsed = DateTime.Now - start;

        Console.WriteLine("\nРезультаты:");
        foreach (var url in links)
        {
            int status = results.GetValueOrDefault(url, -1);
            string color;
            if (status == -1) color = GRAY;
            else if (status >= 200 && status < 300) color = GREEN;
            else if (status >= 300 && status < 400) color = YELLOW;
            else if (status >= 400 && status < 600) color = RED;
            else color = GRAY;
            Console.WriteLine($"  {url} -> {color}{status}{RESET}");
        }

        int total = results.Count;
        int ok = results.Values.Count(s => s >= 200 && s < 300);
        int redirect = results.Values.Count(s => s >= 300 && s < 400);
        int error = results.Values.Count(s => s >= 400 && s < 600);
        int fail = results.Values.Count(s => s == -1);
        Console.WriteLine($"\nСтатистика: Всего: {total}, OK: {ok}, Редиректы: {redirect}, Ошибки: {error}, Сбои: {fail}");
        Console.WriteLine($"Время: {elapsed.TotalSeconds:F2} сек.");

        if (output != null)
        {
            Console.WriteLine($"Экспорт в {output} (не реализован для краткости)");
        }
    }
}
