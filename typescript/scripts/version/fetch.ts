import fs from "fs";
import { execSync } from "child_process";

export const fetchVersion = async (projectVersionPath: string) => {
    const data = JSON.parse(fs.readFileSync(projectVersionPath, "utf-8"));

    data.build = (data.build || 0) + 1;

    try {
        const commit = execSync("git rev-parse HEAD").toString().trim();
        data.lastCommit = commit;
    } catch (err) {
        console.warn("⚠️ Unable to fetch git commit hash. Ensure you are in a git repository.");
    }

    fs.writeFileSync(projectVersionPath, JSON.stringify(data, null, 2));
}

