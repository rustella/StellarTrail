import { getStoredUser } from "./api";

type StoredUser = NonNullable<ReturnType<typeof getStoredUser>>;

export interface AccountProfileView {
  displayName: string;
  avatarUrl: string;
  avatarInitial: string;
  hasNickname: boolean;
  email: string;
  emailText: string;
  hasEmail: boolean;
}

export function buildAccountProfile(loggedIn: boolean): AccountProfileView {
  const user = getStoredUser();
  if (!loggedIn) {
    return emptyAccountProfile();
  }
  if (!user) {
    return {
      displayName: "微信用户",
      avatarUrl: "",
      avatarInitial: "微",
      hasNickname: false,
      email: "",
      emailText: "还没有绑定邮箱",
      hasEmail: false,
    };
  }
  return buildAccountProfileFromUser(user);
}

export function buildAccountProfileFromUser(
  user: StoredUser,
): AccountProfileView {
  const nickname = normalizeProfileNickname(user.nickname);
  const displayName = nickname || user.username || user.email || "微信用户";
  const email = user.email?.trim() ?? "";
  return {
    displayName,
    avatarUrl: user.avatar_url || "",
    avatarInitial: displayName.trim().slice(0, 1) || "微",
    hasNickname: Boolean(nickname),
    email,
    emailText: email ? `已绑定 ${email}` : "还没有绑定邮箱",
    hasEmail: Boolean(email),
  };
}

export function normalizeProfileNickname(value?: string | null): string {
  const nickname = value?.trim() ?? "";
  const defaultNames = ["寻径星野用户", "微信用户", "WeChat User"];
  return nickname && !defaultNames.includes(nickname) ? nickname : "";
}

function emptyAccountProfile(): AccountProfileView {
  return {
    displayName: "",
    avatarUrl: "",
    avatarInitial: "",
    hasNickname: false,
    email: "",
    emailText: "",
    hasEmail: false,
  };
}
