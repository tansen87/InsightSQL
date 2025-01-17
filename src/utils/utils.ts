export function shortFileName(path: string) {
  return path.split("\\").pop().split("/").pop();
}
