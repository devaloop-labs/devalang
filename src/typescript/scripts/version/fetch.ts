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
 * Fetches current version information and increments build number
 */
export async function fetchVersion(): Promise<ProjectVersion> {
  const rootPath = path.resolve(__dirname, '../../../..');
  const projectVersionPath = path.join(rootPath, 'project-version.json');
  
  let projectVersion: ProjectVersion;
  
  if (fs.existsSync(projectVersionPath)) {
    projectVersion = JSON.parse(fs.readFileSync(projectVersionPath, 'utf-8'));
  } else {
    // Fallback to package.json
    const packageJsonPath = path.join(rootPath, 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    projectVersion = {
      version: packageJson.version,
      channel: 'stable',
      build: 1
    };
  }
  
  // Get git commit hash
  try {
    const commit = execSync('git rev-parse --short HEAD', { encoding: 'utf-8' }).trim();
    projectVersion.commit = commit;
  } catch (error) {
    console.warn('⚠️  Could not get git commit hash');
  }
  
  // Increment build number
  projectVersion.build++;
  
  // Save updated version
  fs.writeFileSync(projectVersionPath, JSON.stringify(projectVersion, null, 2) + '\n');
  
  return projectVersion;
}
