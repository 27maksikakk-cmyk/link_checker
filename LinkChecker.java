// LinkChecker.java
import java.io.*;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.util.*;
import java.util.concurrent.*;
import java.util.stream.Collectors;

public class LinkChecker {
    private static final String RESET = "\033[0m";
    private static final String GREEN = "\033[92m";
    private static final String YELLOW = "\033[93m";
    private static final String RED = "\033[91m";
    private static final String GRAY = "\033[90m";

    static class Result {
        String url;
        int status;
        boolean error;
        Result(String url, int status, boolean error) {
            this.url = url; this.status = status; this.error = error;
        }
    }

    public static List<String> loadLinks(String filename) throws IOException {
        List<String> links = new ArrayList<>();
        try (BufferedReader br = new BufferedReader(new FileReader(filename))) {
            String line;
            while ((line = br.readLine()) != null) {
                line = line.trim();
                if (line.isEmpty() || line.startsWith("#")) continue;
                links.add(line);
            }
        }
        return links;
    }

    public static int checkUrl(String url, int timeoutSec, boolean follow) {
        HttpClient client = HttpClient.newBuilder()
                .followRedirects(follow ? HttpClient.Redirect.NORMAL : HttpClient.Redirect.NEVER)
                .connectTimeout(Duration.ofSeconds(timeoutSec))
                .build();
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(url))
                .header("User-Agent", "LinkChecker/1.0")
                .timeout(Duration.ofSeconds(timeoutSec))
                .build();
        try {
            HttpResponse<Void> response = client.send(request, HttpResponse.BodyHandlers.discarding());
            return response.statusCode();
        } catch (Exception e) {
            return -1; // ошибка
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            System.err.println("Использование: java LinkChecker <файл.txt> [--threads N] [--timeout S] [--follow] [--output file]");
            System.exit(1);
        }
        String source = args[0];
        int threads = 4;
        int timeoutSec = 5;
        boolean follow = false;
        String output = null;
        for (int i = 1; i < args.length; i++) {
            if (args[i].equals("--threads") && i+1 < args.length) {
                threads = Integer.parseInt(args[++i]);
            } else if (args[i].equals("--timeout") && i+1 < args.length) {
                timeoutSec = Integer.parseInt(args[++i]);
            } else if (args[i].equals("--follow")) {
                follow = true;
            } else if (args[i].equals("--output") && i+1 < args.length) {
                output = args[++i];
            }
        }

        List<String> links;
        if (source.endsWith(".txt")) {
            links = loadLinks(source);
        } else {
            links = Arrays.asList(args);
        }

        if (links.isEmpty()) {
            System.err.println("Нет ссылок для проверки.");
            System.exit(1);
        }

        System.out.printf("Проверка %d ссылок (потоков: %d, таймаут: %dс)...\n", links.size(), threads, timeoutSec);
        long start = System.currentTimeMillis();

        ExecutorService executor = Executors.newFixedThreadPool(threads);
        List<Future<Result>> futures = new ArrayList<>();
        for (String url : links) {
            futures.add(executor.submit(() -> {
                int status = checkUrl(url, timeoutSec, follow);
                return new Result(url, status, status == -1);
            }));
        }
        List<Result> results = new ArrayList<>();
        for (Future<Result> f : futures) {
            results.add(f.get());
        }
        executor.shutdown();

        long elapsed = System.currentTimeMillis() - start;
        System.out.println("\nРезультаты:");
        for (Result r : results) {
            String color;
            if (r.error) color = GRAY;
            else if (r.status >= 200 && r.status < 300) color = GREEN;
            else if (r.status >= 300 && r.status < 400) color = YELLOW;
            else if (r.status >= 400 && r.status < 600) color = RED;
            else color = GRAY;
            System.out.printf("  %s -> %s%d%s\n", r.url, color, r.status, RESET);
        }

        int total = results.size();
        int ok = 0, redirect = 0, error = 0, fail = 0;
        for (Result r : results) {
            if (r.error) fail++;
            else if (r.status >= 200 && r.status < 300) ok++;
            else if (r.status >= 300 && r.status < 400) redirect++;
            else if (r.status >= 400 && r.status < 600) error++;
        }
        System.out.printf("\nСтатистика: Всего: %d, OK: %d, Редиректы: %d, Ошибки: %d, Сбои: %d\n", total, ok, redirect, error, fail);
        System.out.printf("Время: %.2f сек.\n", elapsed / 1000.0);

        if (output != null) {
            System.out.println("Экспорт в " + output + " (не реализован для краткости)");
        }
    }
}
