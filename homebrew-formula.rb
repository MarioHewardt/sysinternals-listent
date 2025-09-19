# Homebrew Formula Template for listent
# This would be submitted to homebrew-core or maintained in a tap

class Listent < Formula
  desc "Fast command-line tool to discover and list code signing entitlements for macOS executable binaries"
  homepage "https://github.com/sysinternals/listent"
  url "https://github.com/sysinternals/listent/archive/v1.0.0.tar.gz"
  sha256 "TBD" # This would be calculated from the release tarball
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Test that the binary runs and shows help
    assert_match "List entitlements for macOS binaries", shell_output("#{bin}/listent --help")
    
    # Test version output
    assert_match "listent 1.0.0", shell_output("#{bin}/listent --version")
    
    # Test JSON output on an empty directory
    output = shell_output("#{bin}/listent --path /tmp --json")
    json = JSON.parse(output)
    assert json.key?("results")
    assert json.key?("summary")
  end
end