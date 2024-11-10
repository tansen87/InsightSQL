<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { marked } from "marked";
import Prism from "prismjs";
import "prismjs/components/prism-sql";
import "prismjs/themes/prism.css";

const markdownContent = ref(`
### 1.Union
\`\`\`sql
SELECT * FROM _t_1
UNION ALL BY NAME
SELECT * FROM _t_2;
\`\`\`

### 2.Select

\`\`\`sql
SELECT
idx
,name
,SUM(CAST(amount AS FLOAT)) amount
FROM _t_1
WHERE col1 = '0'
OR col2 LIKE 'query%'
LIMIT 10;
\`\`\`

### 3.Join

\`\`\`sql
SELECT * FROM \`file name1\` t1
LEFT JOIN \`file name2\` t2
ON t1.name = t2.name
\`\`\`

### 4.Group by

\`\`\`sql
SELECT 
idx
,SUM(CAST(amount as float)) amount
FROM _t_1
GROUP BY idx
ORDER BY idx DESC
\`\`\`

### 5.Fill null value (COALESCE)

\`\`\`sql
select
sum(cast(coalesce(age, 0) as double)) age
from _t_1
\`\`\`
`);

const compiledMarkdown = ref(marked.parse(markdownContent.value));

onMounted(() => {
  Prism.highlightAll();
});

watch(markdownContent, newContent => {
  compiledMarkdown.value = marked.parse(newContent);
});
</script>

<template>
  <div v-html="compiledMarkdown" />
</template>
