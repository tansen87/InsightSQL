import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export async function viewOpenFile(
  multiple: boolean,
  name: string,
  extensions: string[]
): Promise<string | null> {
  const selected: string = await open({
    multiple: multiple,
    filters: [
      {
        name: name,
        extensions: extensions
      }
    ]
  });

  if (Array.isArray(selected)) {
    return selected.toString();
  } else if (selected === null) {
    return null;
  } else {
    return selected;
  }
}

export async function viewSqlp(path: string, skipRows: string) {
  const result = await invoke("query", {
    path: path,
    sqlQuery: "select * from _t_1 limit 10",
    write: false,
    writeFormat: "csv",
    lowMemory: false,
    skipRows: skipRows,
    schemaLength: "0"
  });

  const q = Array.isArray(result[0]) ? result[0][0] : null;
  if (q.startsWith("Query failed")) {
    throw q;
  }

  const jsonData = JSON.parse(result[0]);
  const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];

  return {
    headerView: Object.keys(arrayData[0]).map(header => ({
      label: header,
      value: header
    })),
    columnView: Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    })),
    dataView: arrayData
  };
}
