// link_checker.cpp
#include <iostream>
#include <string>
#include <vector>
#include <fstream>
#include <thread>
#include <mutex>
#include <curl/curl.h>
#include <chrono>

using namespace std;

struct Result {
    string url;
    long status;
    bool error;
};

size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
    return size * nmemb;
}

long check_url(const string& url, long timeout_sec, bool follow) {
    CURL *curl = curl_easy_init();
    if (!curl) return 0;
    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, timeout_sec);
    curl_easy_setopt(curl, CURLOPT_NOBODY, 1L);
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, follow ? 1L : 0L);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
    curl_easy_setopt(curl, CURLOPT_USERAGENT, "LinkChecker/1.0");
    CURLcode res = curl_easy_perform(curl);
    long http_code = 0;
    if (res == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
    } else {
        http_code = 0;
    }
    curl_easy_cleanup(curl);
    return http_code;
}

vector<string> load_links(const string& filename) {
    vector<string> links;
    ifstream file(filename);
    if (!file.is_open()) {
        cerr << "Не удалось открыть файл: " << filename << endl;
        return links;
    }
    string line;
    while (getline(file, line)) {
        if (line.empty() || line[0] == '#') continue;
        links.push_back(line);
    }
    return links;
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        cerr << "Использование: link_checker <файл.txt> [--threads N] [--timeout S] [--follow] [--output file]" << endl;
        return 1;
    }
    string source = argv[1];
    int threads = 4;
    int timeout_sec = 5;
    bool follow = false;
    string output;
    for (int i = 2; i < argc; ++i) {
        string arg = argv[i];
        if (arg == "--threads" && i+1 < argc) {
            threads = stoi(argv[++i]);
        } else if (arg == "--timeout" && i+1 < argc) {
            timeout_sec = stoi(argv[++i]);
        } else if (arg == "--follow") {
            follow = true;
        } else if (arg == "--output" && i+1 < argc) {
            output = argv[++i];
        }
    }

    curl_global_init(CURL_GLOBAL_DEFAULT);

    vector<string> links;
    if (source.find(".txt") != string::npos) {
        links = load_links(source);
    } else {
        links.push_back(source);
        for (int i = 2; i < argc; ++i) {
            if (string(argv[i])[0] != '-')
                links.push_back(argv[i]);
        }
    }

    if (links.empty()) {
        cerr << "Нет ссылок для проверки." << endl;
        return 1;
    }

    cout << "Проверка " << links.size() << " ссылок (потоков: " << threads << ", таймаут: " << timeout_sec << "с)..." << endl;
    auto start = chrono::steady_clock::now();

    vector<Result> results;
    mutex results_mutex;
    vector<thread> workers;
    int chunk = (links.size() + threads - 1) / threads;
    for (int t = 0; t < threads && t * chunk < (int)links.size(); ++t) {
        int begin = t * chunk;
        int end = min(begin + chunk, (int)links.size());
        workers.emplace_back([&, begin, end]() {
            for (int i = begin; i < end; ++i) {
                string url = links[i];
                long status = check_url(url, timeout_sec, follow);
                lock_guard<mutex> lock(results_mutex);
                results.push_back({url, status, status == 0});
            }
        });
    }
    for (auto& t : workers) t.join();

    auto elapsed = chrono::duration<double>(chrono::steady_clock::now() - start).count();

    cout << "\nРезультаты:" << endl;
    for (const auto& r : results) {
        string color;
        if (r.status >= 200 && r.status < 300) color = "\033[92m";
        else if (r.status >= 300 && r.status < 400) color = "\033[93m";
        else if (r.status >= 400 && r.status < 600) color = "\033[91m";
        else color = "\033[90m";
        cout << "  " << r.url << " -> " << color << r.status << "\033[0m" << endl;
    }

    int total = results.size();
    int ok = 0, redirect = 0, error = 0, fail = 0;
    for (auto& r : results) {
        if (r.status == 0) fail++;
        else if (r.status >= 200 && r.status < 300) ok++;
        else if (r.status >= 300 && r.status < 400) redirect++;
        else if (r.status >= 400 && r.status < 600) error++;
    }
    cout << "\nСтатистика: Всего: " << total << ", OK: " << ok << ", Редиректы: " << redirect
         << ", Ошибки: " << error << ", Сбои: " << fail << endl;
    cout << "Время: " << elapsed << " сек." << endl;

    if (!output.empty()) {
        // экспорт (упрощённо)
        cout << "Экспорт в " << output << " (не реализован для краткости)" << endl;
    }

    curl_global_cleanup();
    return 0;
}
