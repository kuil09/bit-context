class Bitctx < Formula
  desc "Bit-memory context store for AI harness skills"
  homepage "https://github.com/kuil09/bit-context"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.0/bitctx-aarch64-macos"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.0/bitctx-x86_64-macos"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.0/bitctx-aarch64-linux"
      sha256 "PLACEHOLDER_SHA256_ARM64_LINUX"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.0/bitctx-x86_64-linux"
      sha256 "PLACEHOLDER_SHA256_X86_64_LINUX"
    end
  end

  def install
    bin.install Dir["bitctx-*"][0] => "bitctx"
  end

  test do
    system "#{bin}/bitctx", "--version"
    system "#{bin}/bitctx", "--help"
  end
end