import fs from "fs";
import path from "path";
import { execSync } from "child_process";

export const fetchVersion = async (projectVersionPath: string) => {
    // Lire le fichier
    const data = JSON.parse(fs.readFileSync(projectVersionPath, "utf-8"));

    // Incrémenter le numéro de build
    data.build = (data.build || 0) + 1;

    // Récupérer le dernier hash git
    try {
        const commit = execSync("git rev-parse HEAD").toString().trim();
        data.lastCommit = commit;
    } catch (err) {
        console.warn("⚠️ Impossible de récupérer le hash git.");
    }

    // Écrire la mise à jour
    fs.writeFileSync(projectVersionPath, JSON.stringify(data, null, 2));
}

