import fs from "fs";
import path from "path";

export const syncVersion = async (projectVersionPath: string) => {
    const version = fs.readFileSync(projectVersionPath, "utf-8").trim();
    const versionString = JSON.parse(version).version;

    // Package.json
    const pkgPath = path.join(__dirname, "..", "..", "..", "package.json");
    const pkgJson = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
    pkgJson.version = versionString;

    fs.writeFileSync(pkgPath, JSON.stringify(pkgJson, null, 2));

    // Cargo.toml
    const cargoPath = path.join(__dirname, "..", "..", "..", "Cargo.toml");
    const cargoToml = fs.readFileSync(cargoPath, "utf-8");
    const updatedCargo = cargoToml.replace(
        /(version\s*=\s*")[^"]*(")/,
        `$1${versionString}$2`
    );

    fs.writeFileSync(cargoPath, updatedCargo);
}