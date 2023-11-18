import { writable, type Writable } from "svelte/store";

type StoreBuilder = <T>(key: string, defaultValue: T, parser?: ((arg: string | null) => T), serializer?: ((arg: T) => string)) => Writable<T>;

const storageStoreBuilder: (storage: Storage) => StoreBuilder = (storage) => {
    return (key, defaultValue, parser, serializer) => {
        const serialize = serializer || ((x) => JSON.stringify(x));

        const storedValue = storage.getItem(key);
        const initialValue = storedValue === null ? defaultValue : JSON.parse(storedValue);
        const store = writable(initialValue);

        store.subscribe((value) => {
            storage.setItem(key, serialize(value))
        });

        return store;
    };
}

const createLocalStorageStore: StoreBuilder = storageStoreBuilder(localStorage);
const createSessionStorageStore: StoreBuilder = storageStoreBuilder(sessionStorage);

export const documentIdentifier = createSessionStorageStore('documentID', (new Date()).getTime());
export const documentContent = createSessionStorageStore('documentContent', '');
