export default {
  path: "/command",
  redirect: "/command/cmd",
  meta: {
    icon: "command",
    title: "command",
    rank: 1
  },
  children: [
    {
      path: "/command/cmd",
      name: "command",
      component: () => import("@/views/command/cmd.vue"),
      meta: {
        title: "command"
      }
    },
    {
      path: "/command/cat",
      name: "cat",
      component: () => import("@/views/command/cat.vue"),
      meta: {
        title: "cat",
        showLink: false
      }
    },
    {
      path: "/command/excel",
      name: "excel",
      component: () => import("@/views/command/excel.vue"),
      meta: {
        title: "Excel to Csv",
        showLink: false
      }
    }
  ]
} satisfies RouteConfigsTable;
