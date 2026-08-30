class Bitctx < Formula
  desc "Bit-memory context store for AI harness skills"
  homepage "https://github.com/kuil09/bit-context"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.1/bitctx-aarch64-macos"
      sha256 "23574723bd5c0c45002fb4a36c6f95b92f38b58030587679f6dc581fb8ce5d59"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.1/bitctx-x86_64-macos"
      sha256 "b0a1567e02f4f5d26f40f8345b29de1ff3e221203bd87f1a800501ccf9e1099a"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.1/bitctx-aarch64-linux"
      sha256 "f0144e5ec4341a449ebf6f3e8cf198d857b00f6eed34e411791c966256e7b55d"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.2.1/bitctx-x86_64-linux"
      sha256 "3ac001dd46c278e81984431442a846f5bf830991cfb2773979cf65a4a07c9005"
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
