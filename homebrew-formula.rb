# Homebrew Formula Template for listent
# This would be submitted to homebrew-core or maintained in a tap

class Listent < Formula
  desc "Fast Sysinternals command-line tool to discover and list code signing entitlements for macOS executable binaries"
  homepage "https://github.com/mariohewardt/listent"
  url "https://github.com/mariohewardt/listent/archive/v1.0.0.tar.gz"
  sha256 "TBD" # This would be calculated from the release tarball
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def post_install
    # Create configuration directory
    (etc/"listent").mkpath
    
    # Install default daemon configuration if it doesn't exist
    unless (etc/"listent/daemon.toml").exist?
      (etc/"listent/daemon.toml").write <<~EOS
        [daemon]
        polling_interval = 1.0
        auto_start = false

        [monitoring]
        entitlement_filters = []
        path_filters = ["/Applications"]
      EOS
    end
  end

  def caveats
    <<~EOS
      To install the daemon service, run:
        sudo #{bin}/listent install-daemon

      Configuration files are located at:
        #{etc}/listent/daemon.toml

      For daemon management:
        #{bin}/listent daemon-status    # Check status
        #{bin}/listent daemon-stop      # Stop daemon
        #{bin}/listent uninstall-daemon # Uninstall service

      Note: Daemon installation requires administrator privileges.
    EOS
  end

  def uninstall_preflight
    # Stop the daemon if it's running before uninstalling
    if File.exist?("#{bin}/listent")
      # Check if daemon is installed/running
      daemon_status = system("#{bin}/listent", "daemon-status", out: File::NULL, err: File::NULL)
      
      if daemon_status
        ohai "Stopping listent daemon before uninstall..."
        system("sudo", "#{bin}/listent", "daemon-stop", out: File::NULL, err: File::NULL)
        system("sudo", "#{bin}/listent", "uninstall-daemon", out: File::NULL, err: File::NULL)
      end
    end
    
    # Clean up any remaining daemon artifacts
    [
      "/tmp/listent-daemon.sock",
      "/var/run/listent"
    ].each do |path|
      FileUtils.rm_rf(path) if File.exist?(path)
    end
    
    # Attempt to unload any remaining LaunchD services
    system("sudo", "launchctl", "unload", 
           "/Library/LaunchDaemons/com.microsoft.sysinternals.listent.plist",
           out: File::NULL, err: File::NULL)
    system("launchctl", "unload", 
           "#{Dir.home}/Library/LaunchAgents/com.microsoft.sysinternals.listent.plist",
           out: File::NULL, err: File::NULL)
  end

  test do
    # Test that the binary runs and shows help
    assert_match "macOS Entitlement Monitor", shell_output("#{bin}/listent --help")
    
    # Test version output
    assert_match "listent", shell_output("#{bin}/listent --version")
    
    # Test basic scan functionality on empty directory
    require "tempfile"
    Dir.mktmpdir do |tmpdir|
      output = shell_output("#{bin}/listent #{tmpdir} --json --quiet")
      json = JSON.parse(output)
      assert json.key?("results")
      assert json.key?("summary")
      assert_equal 0, json["summary"]["matched"]
    end
    
    # Test daemon commands (without actually installing)
    help_output = shell_output("#{bin}/listent install-daemon --help")
    assert_match "Install daemon service", help_output
    
    # Test entitlement filtering
    help_output = shell_output("#{bin}/listent --help")
    assert_match "entitlement", help_output
  end
end