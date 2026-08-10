// link_checker.go
package main

import (
	"bufio"
	"encoding/csv"
	"encoding/json"
	"flag"
	"fmt"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"
)

const (
	reset  = "\033[0m"
	green  = "\033[92m"
	yellow = "\033[93m"
	red    = "\033[91m"
	gray   = "\033[90m"
)

type result struct {
	url    string
	status int
	err    error
}

func checkURL(url string, timeout time.Duration, follow bool) (int, error) {
	client := &http.Client{
		Timeout: timeout,
		CheckRedirect: func(req *http.Request, via []*http.Request) error {
			if !follow {
				return http.ErrUseLastResponse
			}
			return nil
		},
	}
	resp, err := client.Get(url)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()
	return resp.StatusCode, nil
}

func colorStatus(status int) string {
	if status >= 200 && status < 300 {
		return fmt.Sprintf("%s%d%s", green, status, reset)
	} else if status >= 300 && status < 400 {
		return fmt.Sprintf("%s%d%s", yellow, status, reset)
	} else if status >= 400 && status < 600 {
		return fmt.Sprintf("%s%d%s", red, status, reset)
	}
	return fmt.Sprintf("%d", status)
}

func main() {
	threads := flag.Int("threads", 4, "Количество потоков")
	timeoutSec := flag.Int("timeout", 5, "Таймаут в секундах")
	follow := flag.Bool("follow", false, "Следовать по редиректам")
	output := flag.String("output", "", "Экспорт результатов в JSON или CSV")
	flag.Parse()

	if flag.NArg() == 0 {
		fmt.Println("Использование: link_checker <файл.txt> | <ссылка1> <ссылка2> ...")
		os.Exit(1)
	}

	var links []string
	source := flag.Arg(0)
	if strings.HasSuffix(source, ".txt") {
		file, err := os.Open(source)
		if err != nil {
			fmt.Printf("Ошибка открытия файла: %v\n", err)
			os.Exit(1)
		}
		defer file.Close()
		scanner := bufio.NewScanner(file)
		for scanner.Scan() {
			line := strings.TrimSpace(scanner.Text())
			if line != "" && !strings.HasPrefix(line, "#") {
				links = append(links, line)
			}
		}
	} else {
		links = flag.Args()
	}

	if len(links) == 0 {
		fmt.Println("Нет ссылок для проверки.")
		os.Exit(1)
	}

	fmt.Printf("Проверка %d ссылок (потоков: %d, таймаут: %dс)...\n", len(links), *threads, *timeoutSec)
	start := time.Now()

	var wg sync.WaitGroup
	sem := make(chan struct{}, *threads)
	results := make(map[string]int)
	var mu sync.Mutex
	errs := make(map[string]error)

	for _, url := range links {
		wg.Add(1)
		go func(u string) {
			defer wg.Done()
			sem <- struct{}{}
			defer func() { <-sem }()
			status, err := checkURL(u, time.Duration(*timeoutSec)*time.Second, *follow)
			mu.Lock()
			if err != nil {
				errs[u] = err
				results[u] = 0
			} else {
				results[u] = status
			}
			mu.Unlock()
		}(url)
	}
	wg.Wait()
	elapsed := time.Since(start)

	fmt.Println("\nРезультаты:")
	for _, url := range links {
		status, ok := results[url]
		if !ok || status == 0 {
			fmt.Printf("  %s -> %sERR%s\n", url, gray, reset)
		} else {
			fmt.Printf("  %s -> %s\n", url, colorStatus(status))
		}
	}

	total := len(links)
	okCount := 0
	redirCount := 0
	errCount := 0
	failCount := 0
	for _, s := range results {
		if s == 0 {
			failCount++
		} else if s >= 200 && s < 300 {
			okCount++
		} else if s >= 300 && s < 400 {
			redirCount++
		} else if s >= 400 && s < 600 {
			errCount++
		}
	}
	fmt.Printf("\nСтатистика: Всего: %d, OK: %d, Редиректы: %d, Ошибки: %d, Сбои: %d\n", total, okCount, redirCount, errCount, failCount)
	fmt.Printf("Время: %.2f сек.\n", elapsed.Seconds())

	if *output != "" {
		exportResults(results, *output)
		fmt.Printf("Результаты экспортированы в %s\n", *output)
	}
}

func exportResults(results map[string]int, filename string) {
	ext := filename[len(filename)-4:]
	if ext == ".csv" {
		file, _ := os.Create(filename)
		defer file.Close()
		writer := csv.NewWriter(file)
		defer writer.Flush()
		writer.Write([]string{"URL", "Status"})
		for url, status := range results {
			writer.Write([]string{url, fmt.Sprintf("%d", status)})
		}
	} else if ext == ".son" { // .json
		data, _ := json.MarshalIndent(results, "", "  ")
		os.WriteFile(filename, data, 0644)
	}
}
