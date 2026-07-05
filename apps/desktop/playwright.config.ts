import { defineConfig } from '@playwright/test';
import { existsSync } from 'node:fs';
import path from 'node:path';

function firstExisting(paths: string[]) {
  return paths.find((candidate) => candidate && existsSync(candidate));
}

function chromiumExecutablePath() {
  if (process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH) {
    return process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH;
  }
  if (process.platform === 'win32') {
    return firstExisting([
      path.join(process.env.ProgramFiles ?? '', 'Google/Chrome/Application/chrome.exe'),
      path.join(process.env['ProgramFiles(x86)'] ?? '', 'Google/Chrome/Application/chrome.exe'),
      path.join(process.env.LOCALAPPDATA ?? '', 'Google/Chrome/Application/chrome.exe'),
      path.join(process.env.ProgramFiles ?? '', 'Microsoft/Edge/Application/msedge.exe'),
      path.join(process.env['ProgramFiles(x86)'] ?? '', 'Microsoft/Edge/Application/msedge.exe'),
    ]);
  }
  return firstExisting(['/usr/bin/google-chrome', '/usr/bin/chromium-browser', '/usr/bin/chromium']);
}

const executablePath = chromiumExecutablePath();

export default defineConfig({
  testDir: './src/e2e',
  timeout: 60_000,
  fullyParallel: false,
  use: {
    browserName: 'chromium',
    launchOptions: {
      ...(executablePath ? { executablePath } : {}),
      args: ['--no-sandbox'],
    },
  },
});
