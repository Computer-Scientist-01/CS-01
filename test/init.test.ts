import fs from 'fs';
import path from 'path';
import os from 'os';
import { afterEach, beforeEach, describe, expect, it } from '@jest/globals';
import init from '../command/version-control/init';
import type { InitOptions } from '../command/version-control/init';


let repoRoot: string;
let originalCwd: string;

describe('init', () => {
  beforeEach(() => {
    const tempPrefix = path.join(os.tmpdir(), 'cs01-test-');
    repoRoot = fs.mkdtempSync(tempPrefix);
    originalCwd = process.cwd();
    process.chdir(repoRoot);
  });

  afterEach(() => {

    fs.rmSync(repoRoot, { recursive: true, force: true });
    process.chdir(originalCwd);
  });

  it('should create .CS01/ and all required dirs', () => {
    const success = init();

    expect(success).toBe(true);

    const cs01Dir = path.join(repoRoot, '.CS01');
    expect(fs.existsSync(path.join(cs01Dir, 'objects'))).toBe(true);
    expect(fs.existsSync(path.join(cs01Dir, 'refs'))).toBe(true);
    expect(fs.existsSync(path.join(cs01Dir, 'refs', 'heads'))).toBe(true);
    expectFile(path.join(cs01Dir, 'HEAD'), 'ref: refs/heads/main');
    expectFile(
      path.join(cs01Dir, 'config'),
      '[core]\n  bare = false\n  repositoryformatversion = 0'
    );
  });

  it('should not change anything if init run twice', () => {
    const firstSuccess = init();
    const secondSuccess = init();

    expect(firstSuccess).toBe(true);
    expect(secondSuccess).toBe(false);

    const cs01Dir = path.join(repoRoot, '.CS01');
    expect(fs.existsSync(path.join(cs01Dir, 'objects'))).toBe(true);
    expect(fs.existsSync(path.join(cs01Dir, 'refs'))).toBe(true);
    expect(fs.existsSync(path.join(cs01Dir, 'refs', 'heads'))).toBe(true);
    expectFile(path.join(cs01Dir, 'HEAD'), 'ref: refs/heads/main');
    expectFile(
      path.join(cs01Dir, 'config'),
      '[core]\n  bare = false\n  repositoryformatversion = 0'
    );
  });

  it('should not crash when config is a directory', () => {
    fs.mkdirSync(path.join(repoRoot, 'config'), { recursive: true });
    expect(() => init()).not.toThrow();

    const cs01Dir = path.join(repoRoot, '.CS01');
    expect(fs.existsSync(cs01Dir)).toBe(true);
  });

  describe('bare repos', () => {
    it('should put all CS01 files and folders in root if specify bare', () => {
      const options: InitOptions = { bare: true };
      const success = init(options);

      expect(success).toBe(true);

      expect(fs.existsSync(path.join(repoRoot, 'objects'))).toBe(true);
      expect(fs.existsSync(path.join(repoRoot, 'refs'))).toBe(true);
      expect(fs.existsSync(path.join(repoRoot, 'refs', 'heads'))).toBe(true);
      expectFile(path.join(repoRoot, 'HEAD'), 'ref: refs/heads/main');
      expectFile(
        path.join(repoRoot, 'config'),
        '[core]\n  bare = true\n  repositoryformatversion = 0'
      );

      expect(fs.existsSync(path.join(repoRoot, '.CS01'))).toBe(false);
    });
  });
});

function expectFile(filePath: string, expectedContent: string): void {
  expect(fs.existsSync(filePath)).toBe(true);
  const content = fs.readFileSync(filePath, 'utf8').trim();
  expect(content).toEqual(expectedContent.trim());
}