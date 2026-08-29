class Bitctx < Formula
  desc "Bit-memory context store for AI harness skills"
  homepage "https://github.com/kuil09/bit-context"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.0/bitctx-aarch64-macos"
      sha256 "e7d0415d23f699f6d284d3a38f0a0619f839f5c5cdc73776bbee3dda93c99a38"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.0/bitctx-x86_64-macos"
      sha256 "d1f349dd7979bd12c5cbbb04a6841f57e9005bd44b3853317d1d2ff625f325e1"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.0/bitctx-aarch64-linux"
      sha256 "00e8eabd2dcf8cb98cb7d41d48a453e9f3a152a7864196ba4a57a6fdac3c636f"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.0/bitctx-x86_64-linux"
      sha256 "b9883a180df655b9585bfa93e8f1056500449e4f8c4ae1a7ee4fac178e1aaec1"
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
