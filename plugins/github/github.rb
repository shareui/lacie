require 'json'
require 'net/http'
require 'uri'

$stdout.sync = true

def http_get(url_str)
  uri = URI.parse(url_str)
  http = Net::HTTP.new(uri.host, uri.port)
  http.use_ssl = (uri.scheme == 'https')
  req = Net::HTTP::Get.new(uri.request_uri)
  req['User-Agent'] = 'lacie-github-plugin'
  token = ENV['LACIE_GH_TOKEN']
  if token && !token.strip.empty? && (uri.host == 'github.com' || uri.host == 'api.github.com' || uri.host == 'raw.githubusercontent.com')
    req['Authorization'] = "Bearer #{token.strip}"
  end
  res = http.request(req)
  if res.is_a?(Net::HTTPRedirection) && res['location']
    http_get(res['location'])
  else
    res
  end
end

def check_api_success!(res, action)
  return if res.is_a?(Net::HTTPSuccess)
  token = ENV['LACIE_GH_TOKEN']
  if %w[503 504 403 429].include?(res.code.to_s) && (token.nil? || token.strip.empty?)
    raise "#{action} request failed (#{res.code}). Anonymous API limits may be exhausted; try: export LACIE_GH_TOKEN=\"ghp_...\""
  else
    raise "#{action} request failed: #{res.code}"
  end
end

def handle_search(params)
  query = params['query'].to_s
  url = "https://api.github.com/search/repositories?q=#{URI.encode_www_form_component(query)}"
  res = http_get(url)
  check_api_success!(res, "search")
  data = JSON.parse(res.body)
  items = data['items'] || []
  items.map do |item|
    {
      'storage' => item['full_name'],
      'stars' => item['stargazers_count'] || 0
    }
  end
end

def handle_releases(params)
  storage = params['storage'].to_s
  url = "https://api.github.com/repos/#{storage}/releases"
  res = http_get(url)
  check_api_success!(res, "releases")
  list = JSON.parse(res.body)
  list.map do |r|
    assets = r['assets'] || []
    has_manifest = assets.any? { |a| a['name'] == 'lacie.toml' }
    {
      'tag' => r['tag_name'],
      'pre_release' => r['prerelease'] || false,
      'published_at' => r['published_at'],
      'has_manifest' => has_manifest
    }
  end
end

def handle_manifest(params)
  storage = params['storage'].to_s
  release = params['release'].to_s
  url = "https://raw.githubusercontent.com/#{storage}/#{release}/lacie.toml"
  res = http_get(url)
  return res.body if res.is_a?(Net::HTTPSuccess)

  rel_url = "https://api.github.com/repos/#{storage}/releases/tags/#{release}"
  rel_res = http_get(rel_url)
  if rel_res.is_a?(Net::HTTPSuccess)
    data = JSON.parse(rel_res.body)
    assets = data['assets'] || []
    manifest_asset = assets.find { |a| a['name'] == 'lacie.toml' }
    if manifest_asset && manifest_asset['browser_download_url']
      down_res = http_get(manifest_asset['browser_download_url'])
      return down_res.body if down_res.is_a?(Net::HTTPSuccess)
    end
  elsif %w[503 504 403 429].include?(rel_res.code.to_s)
    token = ENV['LACIE_GH_TOKEN']
    if token.nil? || token.strip.empty?
      raise "manifest request failed (#{rel_res.code}). Anonymous API limits may be exhausted; try: export LACIE_GH_TOKEN=\"ghp_...\""
    end
  end
  raise "manifest lacie.toml not found in #{storage} for release #{release}"
end

def stream_download(url_str, dest, limit = 10)
  raise "too many redirects" if limit == 0
  uri = URI.parse(url_str)
  http = Net::HTTP.new(uri.host, uri.port)
  http.use_ssl = (uri.scheme == 'https')
  req = Net::HTTP::Get.new(uri.request_uri)
  req['User-Agent'] = 'lacie-github-plugin'
  token = ENV['LACIE_GH_TOKEN']
  if token && !token.strip.empty? && uri.host.include?('github.com')
    req['Authorization'] = "Bearer #{token.strip}"
  end
  http.request(req) do |res|
    if res.is_a?(Net::HTTPRedirection) && res['location']
      return stream_download(res['location'], dest, limit - 1)
    end
    raise "download failed: #{res.code}" unless res.is_a?(Net::HTTPSuccess)
    File.open(dest, 'wb') do |f|
      res.read_body do |chunk|
        f.write(chunk)
        f.flush
      end
    end
  end
end

def handle_fetch(params)
  storage = params['storage'].to_s
  release = params['release'].to_s
  file = params['file'].to_s
  dest = params['dest'].to_s

  rel_url = "https://api.github.com/repos/#{storage}/releases/tags/#{release}"
  rel_res = http_get(rel_url)
  check_api_success!(rel_res, "release #{release}")
  data = JSON.parse(rel_res.body)
  assets = data['assets'] || []
  asset = assets.find { |a| a['name'] == file }
  raise "asset #{file} not found in release" unless asset
  down_url = asset['browser_download_url']
  stream_download(down_url, dest)
  { 'success' => true }
end

while (line = $stdin.gets)
  line = line.strip
  next if line.empty?
  begin
    req = JSON.parse(line)
    method = req['method']
    params = req['params'] || {}
    result = case method
             when 'search' then handle_search(params)
             when 'releases' then handle_releases(params)
             when 'manifest' then handle_manifest(params)
             when 'fetch' then handle_fetch(params)
             else raise "unknown method #{method}"
             end
    puts JSON.generate({ 'jsonrpc' => '2.0', 'id' => req['id'], 'result' => result })
  rescue => e
    puts JSON.generate({ 'jsonrpc' => '2.0', 'id' => req ? req['id'] : nil, 'error' => { 'code' => -32603, 'message' => e.message } })
  end
end
