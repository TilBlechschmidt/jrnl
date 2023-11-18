

export function nth(n: number) {
   return ['st', 'nd', 'rd'][((((n + 90) % 100) - 10) % 10) - 1] || 'th';
}

function getCookie(name: string): string | undefined {
   const value = `; ${document.cookie}`;
   const parts = value.split(`; ${name}=`);

   if (parts.length === 2) {
      // @ts-ignore
      return parts.pop().split(';').shift();
   }
}

interface User { 
   sub: string, 
   name: string, 
   preferred_username: string, 
   email: string, 
}

function getUser(): User | undefined {
   const cookieValue = getCookie('user');
   
   if (cookieValue) {
      return JSON.parse(decodeURIComponent(cookieValue));
   }
}

export const user = getUser();