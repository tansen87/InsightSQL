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
        title: "Excel to csv",
        showLink: false
      }
    },
    {
      path: "/command/csv",
      name: "csv",
      component: () => import("@/views/command/csv.vue"),
      meta: {
        title: "csv to xlsx",
        showLink: false
      }
    }
  ]
} satisfies RouteConfigsTable;
