import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

interface ProjectVersion {
  version: string;
  channel: 'stable' | 'preview' | 'beta' | 'alpha';
  build: number;
  commit?: string;
}

/**
 * Bumps the version across all project files
 */
export async function bumpVersion(
  type: 'major' | 'minor' | 'patch' | 'pre',
  preTag: string = 'beta'
): Promise<string> {
  const rootPath = path.resolve(__dirname, '../../../..');
  
  // Read current versions
  const packageJsonPath = path.join(rootPath, 'package.json');
  const cargoTomlPath = path.join(rootPath, 'Cargo.toml');
  const projectVersionPath = path.join(rootPath, 'project-version.json');
  
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  const currentVersion = packageJson.version;
  
  // Parse version
  const versionMatch = currentVersion.match(/^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/);
  if (!versionMatch) {
    throw new Error(`Invalid version format: ${currentVersion}`);
  }
  
  let [, major, minor, patch, pre] = versionMatch;
  let majorNum = parseInt(major);
  let minorNum = parseInt(minor);
  let patchNum = parseInt(patch);
  
  // Calculate new version
  let newVersion: string;
  let channel: ProjectVersion['channel'] = 'stable';
  
  switch (type) {
    case 'major':
      majorNum++;
      minorNum = 0;
      patchNum = 0;
      newVersion = `${majorNum}.${minorNum}.${patchNum}`;
      break;
      
    case 'minor':
      minorNum++;
      patchNum = 0;
      newVersion = `${majorNum}.${minorNum}.${patchNum}`;
      break;
      
    case 'patch':
      patchNum++;
      newVersion = `${majorNum}.${minorNum}.${patchNum}`;
      break;
      
    case 'pre':
      const preVersion = pre ? parseInt(pre.split('.')[1] || '0') + 1 : 0;
      newVersion = `${majorNum}.${minorNum}.${patchNum}-${preTag}.${preVersion}`;
      channel = preTag as ProjectVersion['channel'];
      break;
  }
  
  // Get git commit hash
  let commit: string | undefined;
  try {
    commit = execSync('git rev-parse --short HEAD', { encoding: 'utf-8' }).trim();
  } catch (error) {
    console.warn('⚠️  Could not get git commit hash');
  }
  
  // Update package.json
  packageJson.version = newVersion;
  fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
  console.log(`✅ Updated package.json to ${newVersion}`);
  
  // Update Cargo.toml
  let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
  cargoToml = cargoToml.replace(
    /version = "[\d\.]+-?[a-z\.]*\d*"/,
    `version = "${newVersion}"`
  );
  fs.writeFileSync(cargoTomlPath, cargoToml);
  console.log(`✅ Updated Cargo.toml to ${newVersion}`);
  
  // Update project-version.json
  const projectVersion: ProjectVersion = {
    version: newVersion,
    channel,
    build: 1, // Reset build number on version bump
    commit
  };
  fs.writeFileSync(projectVersionPath, JSON.stringify(projectVersion, null, 2) + '\n');
  console.log(`✅ Updated project-version.json to ${newVersion}`);
  
  // Git commit
  try {
    execSync(`git add ${packageJsonPath} ${cargoTomlPath} ${projectVersionPath}`, { stdio: 'inherit' });
    execSync(`git commit -m "chore: bump version to ${newVersion}"`, { stdio: 'inherit' });
    console.log(`✅ Created git commit for version ${newVersion}`);
  } catch (error) {
    console.warn('⚠️  Could not create git commit (you may need to commit manually)');
  }
  
  return newVersion;
}
