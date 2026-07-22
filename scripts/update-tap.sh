#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"
VERSION_NUM="${VERSION#v}"

DARWIN_ARM64_SHA=$(shasum -a 256 dist/aws-sso-tools_darwin_arm64.tar.gz | awk '{print $1}')
DARWIN_AMD64_SHA=$(shasum -a 256 dist/aws-sso-tools_darwin_amd64.tar.gz | awk '{print $1}')
LINUX_ARM64_SHA=$(shasum -a 256 dist/aws-sso-tools_linux_arm64.tar.gz | awk '{print $1}')
LINUX_AMD64_SHA=$(shasum -a 256 dist/aws-sso-tools_linux_amd64.tar.gz | awk '{print $1}')

cat > /tmp/aws-sso-tools.rb <<RUBY
# typed: false
# frozen_string_literal: true

# This file is generated automatically. DO NOT EDIT.
class AwsSsoTools < Formula
  desc "QoL improvements for engineers using AWS SSO"
  homepage "https://github.com/nathforge/aws-sso-tools"
  version "${VERSION_NUM}"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/nathforge/aws-sso-tools/releases/download/${VERSION}/aws-sso-tools_darwin_arm64.tar.gz"
      sha256 "${DARWIN_ARM64_SHA}"
    end
    if Hardware::CPU.intel?
      url "https://github.com/nathforge/aws-sso-tools/releases/download/${VERSION}/aws-sso-tools_darwin_amd64.tar.gz"
      sha256 "${DARWIN_AMD64_SHA}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/nathforge/aws-sso-tools/releases/download/${VERSION}/aws-sso-tools_linux_arm64.tar.gz"
      sha256 "${LINUX_ARM64_SHA}"
    end
    if Hardware::CPU.intel?
      url "https://github.com/nathforge/aws-sso-tools/releases/download/${VERSION}/aws-sso-tools_linux_amd64.tar.gz"
      sha256 "${LINUX_AMD64_SHA}"
    end
  end

  def install
    bin.install "docker-credential-sso-ecr-login"
    bin.install "aws-sso-should-login"
    bin.install "aws-sso-maybe-login"
    bin.install "aws-sso-login-showing-code"
    bin.install "aws-sso-show-code"
    bin.install "aws-sso-run"
  end
end
RUBY

git clone "https://x-access-token:${GH_TOKEN}@github.com/nathforge/homebrew-tap.git" /tmp/homebrew-tap
cp /tmp/aws-sso-tools.rb /tmp/homebrew-tap/aws-sso-tools.rb
cd /tmp/homebrew-tap
git config user.email "noreply@github.com"
git config user.name "github-actions[bot]"
git add aws-sso-tools.rb
git diff --cached --quiet || git commit -m "Update aws-sso-tools to ${VERSION}"
git push
