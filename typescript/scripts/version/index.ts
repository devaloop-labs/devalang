import path from "path";

import { bumpVersion } from "./bump";
import { syncVersion } from "./sync";
import { fetchVersion } from "./fetch";

const bumpType = process.argv[2] || "patch";

(async () => {
  const projectVersionPath = path.join(__dirname, "../../../project-version.json");

  try {
    const newVersion = await bumpVersion(bumpType, projectVersionPath);

    await fetchVersion(projectVersionPath);
    await syncVersion(projectVersionPath);

    console.log(`✅ Project version updated to : ${newVersion}`);
  }
  catch (error) {
    console.error(`❌ Error updating project version: ${error}`);
    process.exit(1);
  }

})();