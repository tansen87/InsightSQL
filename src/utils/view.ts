import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { shortFileName } from "./utils";

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

export async function trimOpenFile(
  multiple: boolean,
  name: string,
  extensions: string[],
  options?: { includeStatus?: boolean }
): Promise<{
  filePath: string;
  fileInfo: { filename: string; status?: string }[];
}> {
  const selected = await open({
    multiple: multiple,
    filters: [
      {
        name: name,
        extensions: extensions
      }
    ]
  });
  if (Array.isArray(selected)) {
    const filePath = selected.join("|").toString();
    const rows = selected.filter((row: any) => row.trim() !== "");
    const fileInfo = rows.map((file: any) => ({
      filename: shortFileName(file),
      ...(options?.includeStatus ? { status: "" } : {})
    }));
    return { filePath, fileInfo };
  } else if (selected === null) {
    return { filePath: "", fileInfo: [] };
  } else {
    return {
      filePath: selected !== null ? selected : selected,
      fileInfo: [
        {
          filename: shortFileName(selected !== null ? selected : selected),
          status: ""
        }
      ]
    };
  }
}

export async function viewSqlp(path: string, skipRows: string) {
  const result = await invoke("query", {
    path: path,
    sqlQuery: "select * from _t_1 limit 10",
    write: false,
    writeFormat: "csv",
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
    columnView: Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    })),
    dataView: arrayData
  };
}

export async function mapHeaders(path: string, skipRows: string) {
  const headers: string[] = await invoke("map_headers", {
    path: path,
    skipRows: skipRows
  });

  return headers;
}
