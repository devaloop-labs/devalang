import fs from "fs";

export const bumpVersion = async (bumpType: string, projectVersionPath: string) => {
    const versionData = JSON.parse(fs.readFileSync(projectVersionPath, "utf-8"));

    const versionRegex = /^(\d+)\.(\d+)\.(\d+)(?:-([\w.]+))?$/;
    const match = versionData.version.match(versionRegex);

    if (!match) {
        throw new Error("Invalid version format in project-version.json");
    }

    if (!bumpType) {
        console.error("❌ Please specify a version type (major, minor, patch, pre)");
        process.exit(1);
    }

    let [_, major, minor, patch, pre] = match;
    let nextVersion = "";

    switch (bumpType) {
        case "major":
            nextVersion = `${+major + 1}.0.0`;
            break;
        case "minor":
            nextVersion = `${major}.${+minor + 1}.0`;
            break;
        case "patch":
            nextVersion = `${major}.${minor}.${+patch + 1}`;
            break;
        case "pre":
            const [preid, prenumber] = (pre || "alpha.0").split(".");
            nextVersion = `${major}.${minor}.${patch}-${preid}.${+prenumber + 1}`;
            break;
        default:
            console.error("❌ Version type non-recognized (major, minor, patch, pre)");
            process.exit(1);
    }

    versionData.version = nextVersion;
    fs.writeFileSync(projectVersionPath, JSON.stringify(versionData, null, 2));

    return nextVersion;
}