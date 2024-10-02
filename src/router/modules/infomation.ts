export default {
  path: "/infomation",
  redirect: "/infomation/md",
  meta: {
    icon: "informationLine",
    title: "info",
    rank: 3
  },
  children: [
    {
      path: "/infomation/md",
      name: "md",
      component: () => import("@/views/infomation/md.vue"),
      meta: {
        title: "info"
      }
    }
  ]
} as RouteConfigsTable;
