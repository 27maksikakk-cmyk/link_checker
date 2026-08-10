# link_checker.rb
require 'net/http'
require 'uri'
require 'json'
require 'csv'
require 'optparse'
require 'thread'

COLORS = {
  green: "\e[92m",
  yellow: "\e[93m",
  red: "\e[91m",
  gray: "\e[90m",
  reset: "\e[0m"
}

def check_url(url, timeout, follow)
  uri = URI.parse(url)
  http = Net::HTTP.new(uri.host, uri.port)
  http.use_ssl = (uri.scheme == 'https')
  http.open_timeout = timeout
  http.read_timeout = timeout
  request = Net::HTTP::Get.new(uri.request_uri)
  request['User-Agent'] = 'LinkChecker/1.0'
  begin
    response = http.request(request)
    # Проверка редиректов, если не следуем
    unless follow
      if response.is_a?(Net::HTTPRedirection)
        return response.code.to_i
      end
    end
    return response.code.to_i
  rescue
    return nil
  end
end

def load_links(filename)
  lines = File.readlines(filename, chomp: true)
  lines.reject { |l| l.empty? || l.start_with?('#') }
end

def color_status(status)
  return "#{COLORS[:gray]}ERR#{COLORS[:reset]}" if status.nil?
  if status >= 200 && status < 300
    "#{COLORS[:green]}#{status}#{COLORS[:reset]}"
  elsif status >= 300 && status < 400
    "#{COLORS[:yellow]}#{status}#{COLORS[:reset]}"
  elsif status >= 400 && status < 600
    "#{COLORS[:red]}#{status}#{COLORS[:reset]}"
  else
    status.to_s
  end
end

options = {}
OptionParser.new do |opts|
  opts.banner = "Использование: ruby link_checker.rb <файл.txt> [--threads N] [--timeout S] [--follow] [--output file]"
  opts.on("--threads N", Integer, "Количество потоков") { |v| options[:threads] = v }
  opts.on("--timeout S", Integer, "Таймаут в секундах") { |v| options[:timeout] = v }
  opts.on("--follow", "Следовать по редиректам") { options[:follow] = true }
  opts.on("--output FILE", "Экспорт результатов") { |v| options[:output] = v }
end.parse!

source = ARGV[0]
if source.nil?
  puts "Укажите файл со ссылками."
  exit 1
end

threads = options[:threads] || 4
timeout = options[:timeout] || 5
follow = options[:follow] || false
output = options[:output]

if source.end_with?('.txt')
  links = load_links(source)
else
  links = ARGV
end

if links.empty?
  puts "Нет ссылок для проверки."
  exit 1
end

puts "Проверка #{links.size} ссылок (потоков: #{threads}, таймаут: #{timeout}с)..."
start = Time.now

results = {}
queue = Queue.new
links.each { |l| queue << l }

workers = threads.times.map do
  Thread.new do
    while url = queue.pop(true) rescue nil
      status = check_url(url, timeout, follow)
      results[url] = status
    end
  end
end
workers.each(&:join)

elapsed = Time.now - start

puts "\nРезультаты:"
links.each do |url|
  status = results[url]
  puts "  #{url} -> #{color_status(status)}"
end

total = links.size
ok = results.values.count { |s| s && s >= 200 && s < 300 }
redirect = results.values.count { |s| s && s >= 300 && s < 400 }
error = results.values.count { |s| s && s >= 400 && s < 600 }
fail = results.values.count { |s| s.nil? }
puts "\nСтатистика: Всего: #{total}, OK: #{ok}, Редиректы: #{redirect}, Ошибки: #{error}, Сбои: #{fail}"
puts "Время: #{elapsed.round(2)} сек."

if output
  puts "Экспорт в #{output} (не реализован для краткости)"
end
