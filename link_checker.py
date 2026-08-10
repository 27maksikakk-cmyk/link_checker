# link_checker.py
import sys
import argparse
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from urllib.request import Request, urlopen, URLError
from urllib.parse import urlparse
import json
import csv

# Цвета ANSI
GREEN = '\033[92m'
YELLOW = '\033[93m'
RED = '\033[91m'
GRAY = '\033[90m'
RESET = '\033[0m'

def check_url(url, timeout=5, follow_redirects=True):
    """Проверяет URL и возвращает статус-код или None при ошибке."""
    try:
        req = Request(url, headers={'User-Agent': 'LinkChecker/1.0'})
        response = urlopen(req, timeout=timeout)
        status = response.getcode()
        if not follow_redirects and status in (301, 302, 303, 307, 308):
            # Получаем реальный код, но мы можем остановиться
            pass
        return status
    except URLError as e:
        if hasattr(e, 'code'):
            return e.code
        else:
            return None
    except Exception:
        return None

def color_status(status):
    if status is None:
        return f"{GRAY}ERR{RESET}"
    if 200 <= status < 300:
        return f"{GREEN}{status}{RESET}"
    elif 300 <= status < 400:
        return f"{YELLOW}{status}{RESET}"
    elif 400 <= status < 600:
        return f"{RED}{status}{RESET}"
    else:
        return str(status)

def process_links(links, threads=4, timeout=5, follow=False):
    results = {}
    with ThreadPoolExecutor(max_workers=threads) as executor:
        future_to_url = {executor.submit(check_url, url, timeout, follow): url for url in links}
        for future in as_completed(future_to_url):
            url = future_to_url[future]
            try:
                status = future.result()
                results[url] = status
            except Exception:
                results[url] = None
    return results

def load_links_from_file(filename):
    with open(filename, 'r', encoding='utf-8') as f:
        return [line.strip() for line in f if line.strip() and not line.startswith('#')]

def export_results(results, output_file):
    ext = output_file.split('.')[-1].lower()
    if ext == 'json':
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(results, f, indent=2)
    elif ext == 'csv':
        with open(output_file, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow(['URL', 'Status'])
            for url, status in results.items():
                writer.writerow([url, status if status is not None else 'ERROR'])
    else:
        print("Неизвестный формат экспорта. Используйте .json или .csv", file=sys.stderr)

def main():
    parser = argparse.ArgumentParser(description='Проверка ссылок (HTTP-коды)')
    parser.add_argument('source', help='Файл со ссылками или сами ссылки через пробел')
    parser.add_argument('--threads', type=int, default=4, help='Количество потоков (по умолчанию 4)')
    parser.add_argument('--timeout', type=int, default=5, help='Таймаут в секундах (по умолчанию 5)')
    parser.add_argument('--follow', action='store_true', help='Следовать по редиректам')
    parser.add_argument('--output', help='Экспорт результатов в JSON или CSV')
    args = parser.parse_args()

    # Определяем, ссылки это или файл
    if args.source.endswith('.txt'):
        try:
            links = load_links_from_file(args.source)
        except Exception as e:
            print(f"Ошибка чтения файла: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        links = [args.source] + sys.argv[2:]  # оставшиеся аргументы

    if not links:
        print("Нет ссылок для проверки.", file=sys.stderr)
        sys.exit(1)

    print(f"Проверка {len(links)} ссылок (потоков: {args.threads}, таймаут: {args.timeout}с)...")
    start_time = time.time()
    results = process_links(links, threads=args.threads, timeout=args.timeout, follow=args.follow)
    elapsed = time.time() - start_time

    # Вывод результатов в таблице
    print("\nРезультаты:")
    for url, status in results.items():
        color = color_status(status)
        print(f"  {url} -> {color}")

    # Статистика
    total = len(results)
    ok = sum(1 for s in results.values() if s is not None and 200 <= s < 300)
    redirect = sum(1 for s in results.values() if s is not None and 300 <= s < 400)
    error = sum(1 for s in results.values() if s is not None and 400 <= s < 600)
    failed = sum(1 for s in results.values() if s is None)
    print(f"\nСтатистика: Всего: {total}, OK: {ok}, Редиректы: {redirect}, Ошибки: {error}, Сбои: {failed}")
    print(f"Время: {elapsed:.2f} сек.")

    if args.output:
        export_results(results, args.output)
        print(f"Результаты экспортированы в {args.output}")

if __name__ == '__main__':
    main()
