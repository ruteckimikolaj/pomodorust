# This is a Homebrew formula for Pomodorust.
# You would place this file in your Homebrew tap repository.
# For example: your-github-username/homebrew-tap/Formula/pomodorust.rb
class Pomodorust < Formula
  desc "A minimalist, powerful, terminal-based Pomodoro timer written in Rust"
  homepage "https://github.com/ruteckimikolaj/pomodorust"
  version "1.0.0"

  # This section provides different binaries for different macOS architectures.
  if Hardware::CPU.intel?
    # For Intel Macs
    url "https://github.com/ruteckimikolaj/pomodorust/releases/download/v1.0.0/pomodorust-macos-x86_64.tar.gz"
    sha256 "..." # TODO: Update with the SHA256 hash of the tarball
  else
    # For Apple Silicon Macs
    url "https://github.com/ruteckimikolaj/pomodorust/releases/download/v1.0.0/pomodorust-macos-aarch64.tar.gz"
    sha256 "..." # TODO: Update with the SHA256 hash of the tarball
  end

  def install
    # The binary is installed directly from the tarball.
    bin.install "pomodorust"
  end

  # Optional: Add tests to verify the installation.
  test do
    system "#{bin}/pomodorust", "--version"
  end
end
