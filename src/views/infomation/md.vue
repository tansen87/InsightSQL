<script setup lang="ts">
import { useMarkdown } from "@/utils/markdown";

function content() {
  return `
# <u><a href="https://docs.pola.rs/py-polars/html/reference/sql/index.html" target="_blank">Polars SQL Interface</a></u>

### 1.Union
\`\`\`sql
select * from _t_1
union all by name
select * from _t_2;
\`\`\`

### 2.Select

\`\`\`sql
select
idx
,name
,sum(cast(amount as double)) amount
from _t_1
where col1 = '0'
or col2 like 'query%'
limit 10;
\`\`\`

### 3.Join

\`\`\`sql
select * from \`file name1\` t1
left join \`file name2\` t2
on t1.name = t2.name
\`\`\`

### 4.Group by

\`\`\`sql
select
idx
,sum(cast(amount as double)) amount
from _t_1
group by idx
order by idx desc
\`\`\`

### 5.Fill null value (coalesce)

\`\`\`sql
select
sum(cast(coalesce(age, 0) as double)) age
from _t_1
\`\`\`
`;
}

const { compiledMarkdown } = useMarkdown(content);
</script>

<template>
  <div v-html="compiledMarkdown" />
</template>
