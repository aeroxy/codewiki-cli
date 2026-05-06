class CodewikiCli < Formula
  desc "Query GitHub repository wikis via Google Code Wiki from the terminal"
  homepage "https://github.com/aeroxy/codewiki-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/codewiki-cli/releases/download/#{version}/codewiki_macos_arm64.zip"
      sha256 "f5c765bf461638a48a92b34a7b4304a0c9d6c23db73b9f614ae6891e90e16931"
    end
  end

  def install
    bin.install "codewiki"
  end

  test do
    assert_match "codewiki", shell_output("#{bin}/codewiki --help")
  end
end
