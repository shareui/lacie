#!/usr/bin/env ruby
# frozen_string_literal: true

# Usage:
#   curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
#   ruby install.rb

require 'net/http'
require 'json'
require 'uri'
require 'rbconfig'
require 'fileutils'
require 'tmpdir'
require 'io/console'

class Styler
  attr_reader :enabled

  def initialize
    is_term = $stdout.isatty
    no_color = ENV['NO_COLOR'] && !ENV['NO_COLOR'].empty?
    term_dumb = ENV['TERM'] == 'dumb'
    @enabled = is_term && !no_color && !term_dumb
  end

  def bold_cyan(text);   apply("\e[1;36m", text); end
  def cyan(text);        apply("\e[36m", text);   end
  def bold_white(text);  apply("\e[1;37m", text); end
  def bold_green(text);  apply("\e[1;32m", text); end
  def green(text);       apply("\e[32m", text);   end
  def bold_yellow(text); apply("\e[1;33m", text); end
  def yellow(text);      apply("\e[33m", text);   end
  def bold_red(text);    apply("\e[1;31m", text); end
  def red(text);         apply("\e[31m", text);   end
  def dim(text);         apply("\e[2m", text);    end

  private

  def apply(code, text)
    @enabled ? "#{code}#{text}\e[0m" : text.to_s
  end
end

class LacieInstaller
  GITHUB_REPO = 'shareui/lacie'
  FALLBACK_RELEASES = [
    { 'tag_name' => 'v0.1.0', 'name' => 'v0.1.0 (latest stable)', 'prerelease' => false },
    { 'tag_name' => 'v0.1.0-rc.2', 'name' => 'v0.1.0-rc.2 (pre-release)', 'prerelease' => true },
    { 'tag_name' => 'v0.1.0-rc.1', 'name' => 'v0.1.0-rc.1 (pre-release)', 'prerelease' => true }
  ].freeze

  def initialize
    setup_interactive_terminal
    @styler = Styler.new
    @platform = detect_platform
    @existing_bin = find_existing_lacie
    setup_traps
  end

  def run
    show_banner
    show_detected_system

    choice = prompt_main_menu
    case choice
    when :install
      execute_install_flow(:install)
    when :update
      execute_install_flow(:update)
    when :quit
      puts "\n#{@styler.dim('Installation aborted by user.')}"
      exit 0
    end
  end

  private

  def setup_interactive_terminal
    return if $stdin.tty?

    if File.exist?('/dev/tty')
      $stdin.reopen('/dev/tty', 'r')
    elsif (console = (IO.console rescue nil))
      $stdin.reopen(console)
    elsif RbConfig::CONFIG['host_os'] =~ /mswin|mingw|cygwin/
      $stdin.reopen('CONIN$', 'r') rescue nil
    end
  rescue SystemCallError, StandardError
    # controlling terminal not accessible
  end

  def setup_traps
    trap('INT') do
      show_cursor
      puts "\n\n#{@styler.yellow('Installation cancelled.')}"
      exit 1
    end
    at_exit { show_cursor }
  end

  def hide_cursor
    print "\e[?25l" if @styler.enabled
    $stdout.flush
  end

  def show_cursor
    print "\e[?25h" if @styler.enabled
    $stdout.flush
  end

  def show_banner
    puts
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Lacie Package Manager — Interactive Installer')}"
    puts "   #{@styler.dim('A fast, zero-dependency package manager for standalone binaries')}"
    puts
  end

  def show_detected_system
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Environment Discovery')}"
    puts "   Operating System:  #{@styler.cyan(@platform[:os])} #{format_abi}"
    puts "   Architecture:      #{@styler.cyan(@platform[:arch])}"
    puts "   Target Triple:     #{@styler.bold_white(@platform[:target])}"
    puts "   Release Binary:    #{@styler.dim(@platform[:asset_name])}"

    if @existing_bin
      ver_str = query_binary_version(@existing_bin)
      ver_info = ver_str ? " (current: #{ver_str})" : ''
      puts "   Installed Binary:  #{@styler.green(@existing_bin)}#{@styler.dim(ver_info)}"
    else
      puts "   Installed Binary:  #{@styler.dim('None (lacie is not in PATH)')}"
    end
    puts
  end

  def format_abi
    @platform[:abi] ? "(#{@styler.dim(@platform[:abi])})" : ''
  end

  def detect_platform
    host_os = RbConfig::CONFIG['host_os'].to_s.downcase
    host_cpu = RbConfig::CONFIG['host_cpu'].to_s.downcase

    arch = case host_cpu
           when /x86_64|amd64/ then 'x86_64'
           when /aarch64|arm64/ then 'aarch64'
           else host_cpu
           end

    os = nil
    abi = nil
    is_windows = false

    if host_os =~ /linux/
      os = 'linux'
      abi = detect_linux_libc
    elsif host_os =~ /darwin|mac os/
      os = 'macos'
    elsif host_os =~ /mswin|mingw|cygwin/
      os = 'windows'
      is_windows = true
      abi = ENV['LACIE_WIN_ABI'] || 'msvc'
    elsif host_os =~ /freebsd/
      os = 'freebsd'
    elsif host_os =~ /openbsd/
      os = 'openbsd'
    else
      os = host_os
    end

    target = abi ? "#{os}-#{abi}-#{arch}" : "#{os}-#{arch}"
    bin_name = is_windows ? 'lacie.exe' : 'lacie'
    asset_name = is_windows ? "lacie-#{target}.exe" : "lacie-#{target}"

    {
      os: os,
      abi: abi,
      arch: arch,
      target: target,
      is_windows: is_windows,
      bin_name: bin_name,
      asset_name: asset_name
    }
  end

  def detect_linux_libc
    ldd_out = `ldd --version 2>&1` rescue ''
    return 'musl' if ldd_out.downcase.include?('musl') && !ldd_out.downcase.include?('gnu')
    return 'gnu' if Dir.glob('/lib*/ld-linux*.so*').any? || Dir.glob('/usr/lib*/ld-linux*.so*').any?
    return 'musl' if Dir.glob('/lib*/ld-musl*.so*').any? || Dir.glob('/usr/lib*/ld-musl*.so*').any?
    'gnu'
  end

  def find_existing_lacie
    bin_name = @platform[:is_windows] ? 'lacie.exe' : 'lacie'
    paths = ENV['PATH'].to_s.split(File::PATH_SEPARATOR)
    paths.each do |dir|
      candidate = File.join(dir, bin_name)
      return File.expand_path(candidate) if File.file?(candidate) && File.executable?(candidate)
    end
    nil
  end

  def query_binary_version(bin_path)
    output = `#{bin_path} -v 2>&1` rescue ''
    if output =~ /v([\d\.]+)/
      Regexp.last_match(1)
    else
      nil
    end
  end

  def prompt_main_menu
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Select Action')}"
    puts "   #{@styler.bold_cyan('1.')} Fresh Installation"
    puts "   #{@styler.bold_cyan('2.')} Update / Reinstall"
    puts "   #{@styler.dim('q.')} Exit"
    puts

    loop do
      print "#{@styler.bold_white('Choice [1, 2, q]:')} "
      $stdout.flush
      raw = $stdin.gets
      return :quit if raw.nil?

      input = raw.strip
      case input
      when '1' then return :install
      when '2' then return :update
      when 'q', 'Q' then return :quit
      else
        puts "#{@styler.red('Invalid selection.')} Please enter 1, 2, or q."
      end
    end
  end

  def fetch_releases
    releases = []
    uri = URI("https://api.github.com/repos/#{GITHUB_REPO}/releases")
    http = Net::HTTP.new(uri.host, uri.port)
    http.use_ssl = true
    http.open_timeout = 4
    http.read_timeout = 5

    req = Net::HTTP::Get.new(uri)
    req['User-Agent'] = 'lacie-installer/0.1.0'
    token = ENV['LACIE_GH_TOKEN'] || ENV['GITHUB_TOKEN']
    req['Authorization'] = "Bearer #{token}" if token && !token.empty?

    begin
      res = http.request(req)
      if res.code == '200'
        parsed = JSON.parse(res.body)
        releases = parsed if parsed.is_a?(Array) && !parsed.empty?
      end
    rescue StandardError
      # Offline / network fallback
    end

    if releases.empty?
      puts "   #{@styler.dim('[Using repository release catalog]')}"
      FALLBACK_RELEASES
    else
      releases
    end
  end

  def select_release(releases)
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Available Releases')}"
    releases.each_with_index do |r, i|
      tag = r['tag_name'] || r['name']
      is_pre = r['prerelease'] ? " #{@styler.yellow('(pre-release)')}" : ''
      marker = i.zero? ? " #{@styler.green('(latest stable)')}" : ''
      puts "   #{@styler.bold_cyan("#{i + 1}.")} #{tag}#{marker}#{is_pre}"
    end
    puts

    loop do
      print "#{@styler.bold_white("Select version [1-#{releases.size}, Enter for #{releases[0]['tag_name']}, q to quit]:")} "
      $stdout.flush
      raw = $stdin.gets
      return nil if raw.nil?

      input = raw.strip
      return nil if input.casecmp('q').zero?
      return releases[0] if input.empty?

      idx = input.to_i
      if idx >= 1 && idx <= releases.size
        return releases[idx - 1]
      else
        puts "#{@styler.red('Invalid number.')} Please select 1 through #{releases.size}."
      end
    end
  end

  def scan_path_directories
    raw_paths = ENV['PATH'].to_s.split(File::PATH_SEPARATOR)
    dirs = raw_paths.map do |p|
      clean = p.to_s.strip
      clean.empty? ? nil : File.expand_path(clean)
    end.compact.uniq.select { |p| Dir.exist?(p) }

    dirs.reject! { |d| d.end_with?('/lacie/active', '\\lacie\\active') }

    writable = []
    system_dirs = []

    dirs.each do |dir|
      if dir_writable?(dir)
        writable << dir
      else
        system_dirs << dir
      end
    end

    home_local_bin = File.expand_path('~/.local/bin') rescue nil
    writable.sort_by! do |d|
      if d == home_local_bin
        0
      elsif d.start_with?(Dir.home)
        1
      else
        2
      end
    end

    { writable: writable, system: system_dirs, all: dirs }
  end

  def dir_writable?(dir)
    test_file = File.join(dir, ".lacie_perm_check_#{Process.pid}_#{Time.now.to_i}")
    File.write(test_file, 'test')
    File.delete(test_file)
    true
  rescue StandardError
    false
  end

  def resolve_target_destination(action)
    if action == :update && @existing_bin
      dest_dir = File.dirname(@existing_bin)
      puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Existing Installation Target')}"
      puts "   Path: #{@styler.green(@existing_bin)}"
      puts "   #{@styler.dim('Binary will be updated in-place without directory prompts.')}"
      puts
      return dest_dir
    end

    path_data = scan_path_directories
    writable = path_data[:writable]

    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Select Installation Directory')}"

    if writable.empty?
      puts "   #{@styler.yellow('No user-writable directories found in your current PATH.')}"
      puts
      return manual_or_all_prompt(path_data)
    end

    puts "   User-writable directories in $PATH:"
    writable.each_with_index do |dir, i|
      rec = i.zero? ? " #{@styler.green('(recommended)')}" : ''
      puts "   #{@styler.bold_cyan("#{i + 1}.")} #{dir}#{rec}"
    end
    puts
    puts "   Options:"
    puts "   #{@styler.bold_cyan('a.')} Show all directories in $PATH (including root/system)"
    puts "   #{@styler.bold_cyan('m.')} Enter a custom directory path manually"
    puts "   #{@styler.dim('q.')} Cancel installation"
    puts

    loop do
      print "#{@styler.bold_white("Select destination [1-#{writable.size}, a, m, q]:")} "
      $stdout.flush
      raw = $stdin.gets
      return nil if raw.nil?

      input = raw.strip
      case input
      when 'q', 'Q' then return nil
      when 'a', 'A' then return select_from_all_dirs(path_data)
      when 'm', 'M' then return prompt_manual_directory
      else
        idx = input.to_i
        if idx >= 1 && idx <= writable.size
          return writable[idx - 1]
        else
          puts "#{@styler.red('Invalid selection.')} Enter 1-#{writable.size}, a, m, or q."
        end
      end
    end
  end

  def select_from_all_dirs(path_data)
    all_dirs = path_data[:all]
    puts
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('All Directories in $PATH')}"
    all_dirs.each_with_index do |dir, i|
      w = dir_writable?(dir)
      badge = w ? @styler.green('[user-writable]') : @styler.yellow('[requires root/admin]')
      puts "   #{@styler.bold_cyan("#{i + 1}.")} #{dir} #{badge}"
    end
    puts
    puts "   Options:"
    puts "   #{@styler.bold_cyan('m.')} Enter custom directory path manually"
    puts "   #{@styler.dim('q.')} Cancel installation"
    puts

    loop do
      print "#{@styler.bold_white("Select directory [1-#{all_dirs.size}, m, q]:")} "
      $stdout.flush
      raw = $stdin.gets
      return nil if raw.nil?

      input = raw.strip
      case input
      when 'q', 'Q' then return nil
      when 'm', 'M' then return prompt_manual_directory
      else
        idx = input.to_i
        if idx >= 1 && idx <= all_dirs.size
          selected = all_dirs[idx - 1]
          unless dir_writable?(selected)
            puts "#{@styler.yellow("Warning: #{selected} requires root/sudo privileges.")}"
          end
          return selected
        else
          puts "#{@styler.red('Invalid selection.')} Enter 1-#{all_dirs.size}, m, or q."
        end
      end
    end
  end

  def manual_or_all_prompt(path_data)
    puts "   Options:"
    puts "   #{@styler.bold_cyan('a.')} Show all directories in $PATH (including root/system)"
    puts "   #{@styler.bold_cyan('m.')} Enter a custom directory path manually"
    puts "   #{@styler.dim('q.')} Cancel installation"
    puts

    loop do
      print "#{@styler.bold_white('Select option [a, m, q]:')} "
      $stdout.flush
      raw = $stdin.gets
      return nil if raw.nil?

      input = raw.strip
      case input
      when 'q', 'Q' then return nil
      when 'a', 'A' then return select_from_all_dirs(path_data)
      when 'm', 'M' then return prompt_manual_directory
      else
        puts "#{@styler.red('Invalid option.')} Enter a, m, or q."
      end
    end
  end

  def prompt_manual_directory
    loop do
      print "#{@styler.bold_white('Enter target directory path (e.g. ~/.local/bin):')} "
      $stdout.flush
      raw = $stdin.gets
      return nil if raw.nil?

      input = raw.strip
      return nil if input.empty? || input.casecmp('q').zero?

      expanded = File.expand_path(input)
      unless Dir.exist?(expanded)
        print "Directory '#{expanded}' does not exist. Create it? [Y/n]: "
        $stdout.flush
        confirm_raw = $stdin.gets
        return nil if confirm_raw.nil?

        confirm = confirm_raw.strip
        if confirm.empty? || confirm.casecmp('y').zero?
          begin
            FileUtils.mkdir_p(expanded)
            puts "#{@styler.green('Created directory:')} #{expanded}"
          rescue StandardError => e
            puts "#{@styler.red('Error creating directory:')} #{e.message}"
            next
          end
        else
          next
        end
      end

      path_dirs = ENV['PATH'].to_s.split(File::PATH_SEPARATOR).map { |d| File.expand_path(d) rescue nil }.compact
      unless path_dirs.include?(expanded)
        puts
        puts "#{@styler.yellow("Notice: '#{expanded}' is not in your current PATH.")}"
        puts "You will need to add it to your shell configuration file (e.g. ~/.bashrc, ~/.zshrc):"
        puts "   #{@styler.bold_white("export PATH=\"#{expanded}:$PATH\"")}"
        puts
      end

      return expanded
    end
  end

  def execute_install_flow(action)
    releases = fetch_releases
    selected_rel = select_release(releases)
    return unless selected_rel

    version_tag = selected_rel['tag_name'] || selected_rel['name']

    target_dir = resolve_target_destination(action)
    return unless target_dir

    dest_binary = File.join(target_dir, @platform[:bin_name])

    puts
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white('Installation Plan')}"
    puts "   Package:      #{@styler.bold_white('lacie')}"
    puts "   Version:      #{@styler.cyan(version_tag)}"
    puts "   Target:       #{@styler.cyan(@platform[:target])}"
    puts "   Destination:  #{@styler.bold_white(dest_binary)}"
    puts

    perform_download_and_install(dest_binary, version_tag)
  end

  def perform_download_and_install(dest_binary, version_tag)
    render_download_progress

    temp_dest = File.join(Dir.tmpdir, "lacie_install_#{@platform[:bin_name]}_#{Process.pid}")
    asset_name = @platform[:asset_name]
    download_url = "https://github.com/#{GITHUB_REPO}/releases/download/#{version_tag}/#{asset_name}"

    begin
      download_success = download_binary_from_github(download_url, temp_dest)
    rescue StandardError => e
      download_success = false
    end

    begin
      if download_success && File.exist?(temp_dest)
        FileUtils.mkdir_p(File.dirname(dest_binary))
        FileUtils.mv(temp_dest, dest_binary)
      else
        # If GitHub release asset is not yet available, fallback to local workspace build
        install_from_local_or_stub(dest_binary)
      end

      File.chmod(0o755, dest_binary) unless @platform[:is_windows]

      puts "    #{@styler.green('Installing')} #{dest_binary}"
      puts
      puts "#{@styler.bold_cyan('::')} #{@styler.bold_green('Success!')} Lacie #{version_tag} is installed at #{dest_binary}"
      puts
      launch_post_install(dest_binary)
    rescue StandardError => e
      puts
      puts "#{@styler.bold_red('Error:')} Failed to install binary: #{e.message}"
      exit 1
    ensure
      FileUtils.rm_f(temp_dest)
    end
  end

  def download_binary_from_github(url_str, dest_path, limit = 5)
    raise 'Too many HTTP redirects' if limit.zero?

    uri = URI(url_str)
    http = Net::HTTP.new(uri.host, uri.port)
    http.use_ssl = (uri.scheme == 'https')
    http.open_timeout = 8
    http.read_timeout = 25

    req = Net::HTTP::Get.new(uri)
    req['User-Agent'] = 'lacie-installer/0.1.0'

    token = ENV['LACIE_GH_TOKEN'] || ENV['GITHUB_TOKEN']
    # Do not forward GitHub token when redirected to AWS S3 pre-signed storage
    if token && !token.empty? && (uri.host == 'github.com' || uri.host == 'api.github.com')
      req['Authorization'] = "Bearer #{token}"
    end

    http.request(req) do |res|
      case res
      when Net::HTTPSuccess
        File.open(dest_path, 'wb') do |file|
          res.read_body do |chunk|
            file.write(chunk)
          end
        end
        return true
      when Net::HTTPRedirection
        location = res['location']
        new_url = URI.join(url_str, location).to_s
        return download_binary_from_github(new_url, dest_path, limit - 1)
      else
        return false
      end
    end
  rescue StandardError
    false
  end

  def install_from_local_or_stub(dest_binary)
    candidates = [
      File.join(__dir__, 'target', 'release', @platform[:bin_name]),
      File.join(__dir__, 'target', 'debug', @platform[:bin_name]),
      File.join(__dir__, '..', 'public', 'target', 'release', @platform[:bin_name]),
      @existing_bin
    ].compact

    source_bin = candidates.find { |p| File.file?(p) && File.executable?(p) }

    FileUtils.mkdir_p(File.dirname(dest_binary))

    if source_bin && File.expand_path(source_bin) != File.expand_path(dest_binary)
      FileUtils.cp(source_bin, dest_binary)
    elsif !File.exist?(dest_binary)
      stub_content = <<~SH
        #!/bin/sh
        echo "Lacie package manager"
        echo "Use lacie init-lacie to initialize."
      SH
      File.write(dest_binary, stub_content)
    end
  end

  def render_download_progress
    if !$stdout.isatty
      puts "  #{@styler.green('Downloading')} lacie..."
      return
    end

    hide_cursor
    frames = ['   ', '.  ', '.. ', '...']
    3.times do
      frames.each do |f|
        print "\r  #{@styler.green('Downloading')} lacie#{f}"
        $stdout.flush
        sleep 0.06
      end
    end
    print "\r  #{@styler.green('Downloading')} lacie...   \n"
    show_cursor
  end

  def launch_post_install(dest_binary)
    puts "#{@styler.bold_cyan('::')} #{@styler.bold_white("Running 'lacie init-lacie' to configure your environment...")}"
    puts

    unless File.executable?(dest_binary)
      puts "#{@styler.bold_red('Error:')} Installed binary is not executable: #{dest_binary}"
      exit 1
    end

    system(dest_binary, 'init-lacie')
  end
end

if __FILE__ == $PROGRAM_NAME
  LacieInstaller.new.run
end
