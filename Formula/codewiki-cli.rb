class CodewikiCli < Formula
  desc "Query GitHub repository wikis via Google Code Wiki from the terminal"
  homepage "https://github.com/aeroxy/codewiki-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/codewiki-cli/releases/download/#{version}/codewiki_macos_arm64.zip"
      sha256 "f4e68d70f9d17cf72c51d06001f5e163f855fb9f0baedc8057ca15f53c1111d1"
    end
  end

  def install
    bin.install "codewiki"
  end

  test do
    assert_match "codewiki", shell_output("#{bin}/codewiki --help")
  end
end
