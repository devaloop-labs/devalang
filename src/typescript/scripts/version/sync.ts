import * as fs from 'fs';
import * as path from 'path';

/**
 * Synchronizes version from package.json to all other version files
 */
export async function syncVersion(): Promise<void> {
  const rootPath = path.resolve(__dirname, '../../../..');
  
  // Read package.json as source of truth
  const packageJsonPath = path.join(rootPath, 'package.json');
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  const version = packageJson.version;
  
  if (!version) {
    throw new Error('No version found in package.json');
  }
  
  // Determine channel from version
  const isPreRelease = version.includes('-');
  let channel: 'stable' | 'preview' | 'beta' | 'alpha' = 'stable';
  
  if (isPreRelease) {
    if (version.includes('alpha')) channel = 'alpha';
    else if (version.includes('beta')) channel = 'beta';
    else channel = 'preview';
  }
  
  // Update Cargo.toml
  const cargoTomlPath = path.join(rootPath, 'Cargo.toml');
  let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
  cargoToml = cargoToml.replace(
    /version = "[\d\.]+-?[a-z\.]*\d*"/,
    `version = "${version}"`
  );
  fs.writeFileSync(cargoTomlPath, cargoToml);
  console.log(`✅ Synchronized Cargo.toml to ${version}`);
  
  // Update project-version.json
  const projectVersionPath = path.join(rootPath, 'project-version.json');
  let projectVersion: any = { version, channel, build: 1 };
  
  if (fs.existsSync(projectVersionPath)) {
    const existing = JSON.parse(fs.readFileSync(projectVersionPath, 'utf-8'));
    projectVersion.build = existing.build || 1;
    projectVersion.commit = existing.commit;
  }
  
  projectVersion.version = version;
  projectVersion.channel = channel;
  
  fs.writeFileSync(projectVersionPath, JSON.stringify(projectVersion, null, 2) + '\n');
  console.log(`✅ Synchronized project-version.json to ${version}`);
  
  // Update installer/windows/devalang.wxs (WiX installer)
  const wixFilePath = path.join(rootPath, 'installer', 'windows', 'devalang.wxs');
  if (fs.existsSync(wixFilePath)) {
    try {
      let wixContent = fs.readFileSync(wixFilePath, 'utf-8');
      
      // Update ProductVersion in the <?define ?> section
      wixContent = wixContent.replace(
        /<\?define ProductVersion="[\d\.]+\-?[a-z\.]*\d*" \?>/,
        `<` + `?define ProductVersion="${version}" ?` + `>`
      );
      
      fs.writeFileSync(wixFilePath, wixContent);
      console.log(`✅ Synchronized installer/windows/devalang.wxs to ${version}`);
    } catch (error) {
      console.warn('⚠️  Could not update installer/windows/devalang.wxs:', error);
    }
  } else {
    console.warn('⚠️  installer/windows/devalang.wxs not found, skipping');
  }
}
