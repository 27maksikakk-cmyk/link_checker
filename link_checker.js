// link_checker.js
const axios = require('axios');
const fs = require('fs');
const yargs = require('yargs');
const { hideBin } = require('yargs/helpers');

const colors = {
  green: '\x1b[92m',
  yellow: '\x1b[93m',
  red: '\x1b[91m',
  gray: '\x1b[90m',
  reset: '\x1b[0m'
};

async function checkUrl(url, timeout, follow) {
  try {
    const config = {
      timeout: timeout * 1000,
      headers: { 'User-Agent': 'LinkChecker/1.0' },
      maxRedirects: follow ? 5 : 0,
    };
    const response = await axios.get(url, config);
    return response.status;
  } catch (error) {
    if (error.response) {
      return error.response.status;
    } else if (error.request) {
      return null; // no response
    } else {
      return null;
    }
  }
}

function loadLinks(filename) {
  const content = fs.readFileSync(filename, 'utf8');
  return content.split('\n')
    .map(line => line.trim())
    .filter(line => line && !line.startsWith('#'));
}

function colorStatus(status) {
  if (status === null) {
    return `${colors.gray}ERR${colors.reset}`;
  }
  if (status >= 200 && status < 300) {
    return `${colors.green}${status}${colors.reset}`;
  } else if (status >= 300 && status < 400) {
    return `${colors.yellow}${status}${colors.reset}`;
  } else if (status >= 400 && status < 600) {
    return `${colors.red}${status}${colors.reset}`;
  } else {
    return `${status}`;
  }
}

async function main() {
  const argv = yargs(hideBin(process.argv))
    .usage('Использование: $0 <файл.txt> [--threads N] [--timeout S] [--follow] [--output file]')
    .option('threads', { type: 'number', description: 'Количество потоков', default: 4 })
    .option('timeout', { type: 'number', description: 'Таймаут в секундах', default: 5 })
    .option('follow', { type: 'boolean', description: 'Следовать по редиректам' })
    .option('output', { type: 'string', description: 'Экспорт результатов' })
    .help()
    .parse();

  const source = argv._[0];
  if (!source) {
    console.error('Укажите файл со ссылками.');
    process.exit(1);
  }

  let links;
  if (source.endsWith('.txt')) {
    links = loadLinks(source);
  } else {
    links = argv._;
  }

  if (!links || links.length === 0) {
    console.error('Нет ссылок для проверки.');
    process.exit(1);
  }

  console.log(`Проверка ${links.length} ссылок (потоков: ${argv.threads}, таймаут: ${argv.timeout}с)...`);
  const start = Date.now();

  // ограничение параллелизма
  const concurrency = argv.threads;
  const results = {};
  const queue = [...links];
  const workers = [];

  async function worker() {
    while (queue.length > 0) {
      const url = queue.shift();
      const status = await checkUrl(url, argv.timeout, argv.follow);
      results[url] = status;
    }
  }

  const workerPromises = [];
  for (let i = 0; i < Math.min(concurrency, links.length); i++) {
    workerPromises.push(worker());
  }
  await Promise.all(workerPromises);

  const elapsed = (Date.now() - start) / 1000;

  console.log('\nРезультаты:');
  for (const url of links) {
    const status = results[url];
    console.log(`  ${url} -> ${colorStatus(status)}`);
  }

  const total = links.length;
  const ok = Object.values(results).filter(s => s !== null && s >= 200 && s < 300).length;
  const redirect = Object.values(results).filter(s => s !== null && s >= 300 && s < 400).length;
  const error = Object.values(results).filter(s => s !== null && s >= 400 && s < 600).length;
  const fail = Object.values(results).filter(s => s === null).length;
  console.log(`\nСтатистика: Всего: ${total}, OK: ${ok}, Редиректы: ${redirect}, Ошибки: ${error}, Сбои: ${fail}`);
  console.log(`Время: ${elapsed.toFixed(2)} сек.`);

  if (argv.output) {
    console.log(`Экспорт в ${argv.output} (не реализован для краткости)`);
  }
}

main().catch(console.error);
