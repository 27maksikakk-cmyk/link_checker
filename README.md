🔗 Проверка ссылок (HTTP-коды) — быстрый и надёжный
Версия: 1.0.0 | Лицензия: MIT | Статус: ✅ Активная разработка

https://img.shields.io/github/repo-size/yourusername/link-checker https://img.shields.io/github/last-commit/yourusername/link-checker https://img.shields.io/github/languages/count/yourusername/link-checker

🌐 Описание
Проверка ссылок (HTTP-коды) — это консольная утилита для массовой проверки HTTP-статусов веб-страниц. Программа позволяет быстро выявить битые ссылки, перенаправления и другие проблемы с доступностью ресурсов.

Возможности:

✅ Проверка произвольного количества ссылок (из аргументов или файла)

✅ Отображение HTTP-статус-кодов с цветовой индикацией (2xx/3xx/4xx/5xx)

✅ Многопоточная обработка (ускорение проверки)

✅ Настраиваемый таймаут и количество повторных попыток

✅ Следование по перенаправлениям (опционально)

✅ Вывод результатов в виде таблицы или списка

✅ Экспорт отчёта в CSV/JSON (опционально)

✅ Кроссплатформенность (Linux, macOS, Windows)

Проект содержит 8 полноценных реализаций на разных языках программирования. Все версии используют HTTP-клиенты и предоставляют единый интерфейс командной строки.

✨ Возможности
Функция	Описание
Проверка HTTP-статусов	Получение статус-кода для каждой ссылки
Цветовая индикация	Зелёный – успех (2xx), жёлтый – перенаправление (3xx), красный – ошибка (4xx/5xx), серый – таймаут/сбой
Многопоточность	Параллельная проверка для ускорения работы
Чтение из файла	Поддержка списка ссылок в текстовом файле (по одной на строку)
Следование по редиректам	Автоматическое следование (опционально)
Таймаут	Настройка максимального времени ожидания ответа
Экспорт	Сохранение результатов в CSV или JSON
Кроссплатформенность	Работает на всех основных ОС
📦 Установка и запуск
Каждая реализация находится в отдельной папке. Для запуска требуется соответствующий компилятор/интерпретатор.

Язык	Файл	Зависимости	Команда запуска
Python	link_checker.py	requests (опционально, можно и urllib)	pip install requests && python3 link_checker.py urls.txt
Go	link_checker.go	нет (встроенный net/http)	go run link_checker.go urls.txt
Rust	link_checker.rs	reqwest, tokio, clap	cargo run -- urls.txt
C++	link_checker.cpp	libcurl, nlohmann/json	g++ -std=c++17 -o link_checker link_checker.cpp -lcurl && ./link_checker urls.txt
Java	LinkChecker.java	java.net.http (Java 11+)	javac LinkChecker.java && java LinkChecker urls.txt
C#	link_checker.cs	System.Net.Http	dotnet run urls.txt
Ruby	link_checker.rb	net/http, json (встроены)	ruby link_checker.rb urls.txt
Node.js	link_checker.js	axios, yargs	npm install axios yargs && node link_checker.js urls.txt
📂 Структура репозитория
text
.
├── README.md
├── python/
│   └── link_checker.py
├── go/
│   └── link_checker.go
├── rust/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── cpp/
│   └── link_checker.cpp
├── java/
│   └── LinkChecker.java
├── csharp/
│   └── link_checker.cs
├── ruby/
│   └── link_checker.rb
└── javascript/
    ├── package.json
    └── link_checker.js
🎮 Использование
bash
# Проверка ссылок из файла
link_checker urls.txt

# Проверка ссылок из аргументов командной строки
link_checker https://example.com https://google.com

# С многопоточностью (8 потоков)
link_checker urls.txt --threads 8

# Таймаут 3 секунды
link_checker urls.txt --timeout 3

# Следование по редиректам
link_checker urls.txt --follow

# Экспорт в CSV
link_checker urls.txt --output report.csv

# Экспорт в JSON
link_checker urls.txt --output report.json
🛠️ Особенности реализаций
Python – использует requests для удобной работы с HTTP и concurrent.futures для многопоточности.

Go – встроенный net/http и sync.WaitGroup – быстрая и эффективная реализация.

Rust – reqwest и tokio для асинхронного параллельного выполнения.

C++ – libcurl для HTTP-запросов и потоки C++11.

Java – HttpClient (Java 11+) и ExecutorService для пула потоков.

C# – HttpClient и Task для асинхронной обработки.

Ruby – Net::HTTP и Thread для параллельных запросов.

Node.js – axios и Promise.all для асинхронной проверки.

Все версии поддерживают цветной вывод в терминале, настройку таймаута и количества потоков, а также экспорт результатов.

🤝 Вклад
PR и issues приветствуются. Добавляйте поддержку других форматов, улучшайте производительность, расширяйте функциональность.

📄 Лицензия
MIT License.
