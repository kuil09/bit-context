class Bitctx < Formula
  desc "Bit-memory context store for AI harness skills"
  homepage "https://github.com/kuil09/bit-context"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.6/bitctx-aarch64-macos"
      sha256 "bf3782a7ff256d1d1670522b1c85e3d4927d64f6ab3895212f7b82b084c37ae7"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.6/bitctx-x86_64-macos"
      sha256 "853763d2a6d678dcb8772d037c825b63e079373e2ef1df7a0841784e31f0b04b"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.6/bitctx-aarch64-linux"
      sha256 "b1f6624db1dd7e9cd3281679536b3fb918fc966a7d95a1acb95c29734a58a547"
    else
      url "https://github.com/kuil09/bit-context/releases/download/v0.1.6/bitctx-x86_64-linux"
      sha256 "2d036ce4589f3181f4ac0dda7ff90c49057b0cc905a59c53ada20ed5fa5a3fea"
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