App<IAppOption>({
  globalData: {
    apiBaseUrl: "http://127.0.0.1:8080",
  },
});

interface IAppOption {
  globalData: {
    apiBaseUrl: string;
  };
}
